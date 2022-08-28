use bevy::{prelude::*, time::Stopwatch};

use crate::GameState;

use super::PlayingState;

#[derive(Component)]
pub(crate) struct GameTag;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>()
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Playing).with_system(you_lost))
            .add_system_set(
                SystemSet::on_exit(GameState::Playing).with_system(despawn_all_the_things),
            );
    }
}

#[derive(Default)]
pub(crate) struct Stats {
    pub(crate) life: u32,
    pub(crate) time: Stopwatch,
    pub(crate) credits: u32,
    pub(crate) killed: u32,
}

fn setup(mut commands: Commands, mut state: ResMut<State<PlayingState>>) {
    let _ = state.overwrite_set(PlayingState::Playing);

    commands.insert_resource(Stats {
        life: 20,
        time: Stopwatch::new(),
        credits: 50,
        killed: 0,
    });
}

fn you_lost(mut state: ResMut<Stats>, mut game_state: ResMut<State<GameState>>, time: Res<Time>) {
    if state.life == 0 {
        warn!("you lost!");
        game_state.set(GameState::Lost).unwrap();
    }
    state.time.tick(time.delta());
}

fn despawn_all_the_things(
    mut commands: Commands,
    entities: Query<Entity, With<GameTag>>,
    mut state: ResMut<State<PlayingState>>,
) {
    let _ = state.overwrite_set(PlayingState::Playing);
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}
