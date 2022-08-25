use bevy::prelude::*;

use crate::GameState;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Playing).with_system(move_zombies));
    }
}

#[derive(Component)]
pub(crate) struct Zombie {}

fn move_zombies(
    mut commands: Commands,
    mut zombies: Query<(Entity, &mut Transform), With<Zombie>>,
    time: Res<Time>,
) {
    for (entity, mut zombie) in &mut zombies {
        let tr = zombie.translation;
        zombie.translation += (Vec3::ZERO - tr).normalize() * time.delta_seconds();
        if zombie.translation.distance_squared(Vec3::ZERO) < 0.01 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
