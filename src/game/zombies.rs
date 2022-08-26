use bevy::prelude::*;

use crate::{
    game::terrain_spawner::{map_to_world, world_to_map},
    GameState,
};

use super::terrain_spawner::Pathfinding;

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
pub(crate) struct IdleZombie;

#[derive(Component)]
pub(crate) struct Zombie {
    pub(crate) path: polyanya::Path,
    pub(crate) current_path: usize,
}

fn move_zombies(
    mut commands: Commands,
    mut zombies: Query<(Entity, &mut Transform, &Zombie)>,
    time: Res<Time>,
) {
    for (entity, mut transform, zombie) in &mut zombies {
        let tr = transform.translation;
        let target = zombie.path.path[zombie.current_path];
        let target = Vec3::new(target.x, 0.0, target.y);
        transform.translation += (target - tr).normalize() * time.delta_seconds() * 0.25;
        if transform.translation.distance_squared(Vec3::ZERO) < 0.01 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn refresh_zombie_path(
    mut commands: Commands,
    idle_zombies: Query<(Entity, &Transform), With<IdleZombie>>,
    pathed_zombies: Query<Entity, (With<Zombie>, Without<IdleZombie>)>,
    mut zombies: Query<(&Transform, &mut Zombie)>,
    pathfinding: Res<Pathfinding>,
) {
    let mut max_per_turn = 5;
    for (zombie, transform) in &idle_zombies {
        let map = world_to_map(Vec2::new(transform.translation.x, transform.translation.z));
        let world = map_to_world(map);
        commands
            .entity(zombie)
            .insert(Zombie {
                path: pathfinding.mesh.path(world, Vec2::ZERO),
                current_path: 0,
            })
            .remove::<IdleZombie>();
        max_per_turn -= 1;
        if max_per_turn == 0 {
            return;
        }
    }
    if pathfinding.is_changed() {
        for entity in &pathed_zombies {
            commands
                .entity(entity)
                .remove::<Zombie>()
                .insert(IdleZombie);
        }
    } else {
        for (transform, mut zombie) in &mut zombies {
            let target = zombie.path.path[zombie.current_path];
            let target = Vec3::new(target.x, 0.0, target.y);
            if transform.translation.distance_squared(target) < 0.01 {
                zombie.current_path += 1;
            }
        }
    }
}
