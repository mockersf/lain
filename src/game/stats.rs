use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
pub(crate) struct GameTag;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Stats { life: 0 })
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(you_lost))
            .add_system_set(
                SystemSet::on_exit(GameState::Playing).with_system(despawn_all_the_things),
            );
    }
}

pub(crate) struct Stats {
    pub(crate) life: u32,
}

fn setup(mut commands: Commands) {
    commands.insert_resource(Stats { life: 20 });
}

fn you_lost(state: Res<Stats>, mut game_state: ResMut<State<GameState>>) {
    if state.life == 0 {
        warn!("you lost!");
        game_state.set(GameState::Lost).unwrap();
    }
}

fn despawn_all_the_things(mut commands: Commands, entities: Query<Entity, With<GameTag>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}
