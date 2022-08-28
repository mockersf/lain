use bevy::prelude::*;

use crate::{assets::SceneryAssets, GameState};

use super::{stats::GameTag, terra::Plane, zombies::Zombie, PlayingState};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(trigger_attack)
                .with_system(move_missiles),
        );
    }
}

#[derive(Component)]
pub(crate) struct Tower {
    pub(crate) timer: Timer,
    pub(crate) strength: f32,
    pub(crate) plane: Plane,
}

#[derive(Component)]
pub(crate) struct Missile {
    pub(crate) strength: f32,
    pub(crate) plane: Plane,
    pub(crate) target: Entity,
}

fn trigger_attack(
    mut commands: Commands,
    zombies: Query<(Entity, &Transform, &Zombie)>,
    mut towers: Query<(&mut Tower, &Transform)>,
    time: Res<Time>,
    playing_state: Res<State<PlayingState>>,
    plane: Res<Plane>,
    scenery: Res<SceneryAssets>,
) {
    if *playing_state.current() != PlayingState::SwitchingPlane {
        for (mut tower, tt) in &mut towers {
            if tower.timer.tick(time.delta()).just_finished() {
                let mut to_attack = None;
                for (ze, zt, zombie) in &zombies {
                    if zombie.plane == tower.plane
                        && zt.translation.distance_squared(tt.translation) < 4.0
                    {
                        to_attack = Some(ze);
                        break;
                    }
                }
                if let Some(entity_to_attack) = to_attack {
                    commands
                        .spawn_bundle(PbrBundle {
                            mesh: scenery.missile_mesh.clone_weak(),
                            material: scenery.missile_material.clone_weak(),
                            transform: Transform::from_translation(Vec3::new(
                                tt.translation.x,
                                0.5,
                                tt.translation.z,
                            )),
                            visibility: Visibility {
                                is_visible: *plane == tower.plane,
                            },
                            ..default()
                        })
                        .insert_bundle((
                            Missile {
                                strength: tower.strength,
                                plane: tower.plane,
                                target: entity_to_attack,
                            },
                            GameTag,
                        ));
                }
            }
        }
    }
}

fn move_missiles(
    mut commands: Commands,
    mut missiles: Query<(Entity, &mut Transform, &Missile)>,
    mut zombies: Query<(&Transform, &mut Zombie), Without<Missile>>,
    time: Res<Time>,
    playing_state: Res<State<PlayingState>>,
) {
    if *playing_state.current() != PlayingState::SwitchingPlane {
        for (entity, mut transform, missile) in &mut missiles {
            if let Ok((target, mut zombie)) = zombies.get_mut(missile.target) {
                let tr = transform.translation;
                transform.translation +=
                    (target.translation - tr).normalize() * time.delta_seconds() * 2.0;

                if transform.translation.distance_squared(target.translation) < 0.01 {
                    commands.entity(entity).despawn();
                    zombie.life -= missile.strength;
                }
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
