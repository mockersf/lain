use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    game::terrain_spawner::{map_to_world, world_to_map},
    GameState,
};

use super::{stats::Stats, terra::Plane, terrain_spawner::Pathfinding, PlayingState};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(move_zombies)
                .with_system(refresh_zombie_path.before(move_zombies)),
        );
    }
}

#[derive(Component)]
pub(crate) struct IdleZombie {
    pub(crate) plane: Plane,
}

#[derive(Component)]
pub(crate) struct Zombie {
    pub(crate) path: polyanya::Path,
    pub(crate) current_path: usize,
    pub(crate) plane: Plane,
}

fn move_zombies(
    mut commands: Commands,
    mut zombies: Query<(Entity, &mut Transform, &Zombie)>,
    time: Res<Time>,
    mut stats: ResMut<Stats>,
    playing_state: Res<State<PlayingState>>,
) {
    if *playing_state.current() != PlayingState::SwitchingPlane {
        for (entity, mut transform, zombie) in &mut zombies {
            let tr = transform.translation;
            if zombie.current_path < zombie.path.path.len() {
                let target = zombie.path.path[zombie.current_path];
                let target = Vec3::new(target.x, 0.0, target.y);
                transform.look_at(target, Vec3::Y);
                transform.rotate(Quat::from_rotation_y(PI));
                transform.translation += (target - tr).normalize() * time.delta_seconds() * 0.25;
                if transform.translation.distance_squared(Vec3::ZERO) < 0.01 {
                    commands.entity(entity).despawn_recursive();
                    if let Some(life) = stats.life.checked_sub(1) {
                        stats.life = life;
                    }
                }
            }
        }
    }
}

fn refresh_zombie_path(
    mut commands: Commands,
    idle_zombies: Query<(Entity, &Transform, &IdleZombie)>,
    mut zombies: Query<(Entity, &Transform, &mut Zombie), Without<IdleZombie>>,
    pathfinding: Res<Pathfinding>,
) {
    let mut max_per_turn = 5;
    for (zombie, transform, idle) in &idle_zombies {
        let map = world_to_map(Vec2::new(transform.translation.x, transform.translation.z));
        let world = map_to_world(map);
        let path = pathfinding.mesh.path(world, Vec2::ZERO);
        if !path.path.is_empty() {
            commands.entity(zombie).insert(Zombie {
                path,
                current_path: 0,
                plane: idle.plane,
            });
        }
        commands.entity(zombie).remove::<IdleZombie>();

        max_per_turn -= 1;
        if max_per_turn == 0 {
            return;
        }
    }
    if pathfinding.is_changed() {
        for (entity, _, zombie) in &zombies {
            commands
                .entity(entity)
                .remove::<Zombie>()
                .insert(IdleZombie {
                    plane: zombie.plane,
                });
        }
    } else {
        for (entity, transform, mut zombie) in &mut zombies {
            let target = zombie.path.path[zombie.current_path];
            let target = Vec3::new(target.x, 0.0, target.y);
            if transform.translation.distance_squared(target) < 0.01 {
                zombie.current_path += 1;
                if zombie.current_path == zombie.path.path.len() {
                    commands
                        .entity(entity)
                        .remove::<Zombie>()
                        .insert(IdleZombie {
                            plane: zombie.plane,
                        });
                }
            }
        }
    }
}
