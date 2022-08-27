use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{assets::ZombieAssets, GameState};

use super::{terrain_spawner::map_to_world, zombies::IdleZombie};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Playing).with_system(spawn_zombies));
    }
}

#[derive(Component)]
pub(crate) struct ZombieNest {
    pub(crate) map: IVec2,
    pub(crate) lot: IVec2,
    pub(crate) timer: Timer,
}

fn spawn_zombies(
    mut commands: Commands,
    mut nests: Query<&mut ZombieNest>,
    zombie_assets: Res<ZombieAssets>,
    time: Res<Time>,
) {
    for mut nest in &mut nests {
        if nest.timer.tick(time.delta()).just_finished() {
            let position = map_to_world((nest.map, nest.lot));

            let mut transform = Transform::from_xyz(position.x, 0.2, position.y)
                .looking_at(Vec3::ZERO, Vec3::Y)
                .with_scale(Vec3::splat(0.05));
            transform.rotate(Quat::from_rotation_y(PI));
            commands
                .spawn_bundle(SceneBundle {
                    scene: zombie_assets.zombie.clone_weak(),
                    transform,
                    ..default()
                })
                .insert(IdleZombie);
        }
    }
}
