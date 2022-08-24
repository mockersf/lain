use std::f32::consts::PI;

use bevy::{
    prelude::{
        App, Color, Commands, DespawnRecursiveExt, DirectionalLight, Entity, Quat, Query, Res,
        ResMut, State, SystemSet, Transform, Vec3,
    },
    time::{Time, Timer},
};
use bevy_easings::{EaseFunction, EaseValue, Lerp};
use interpolation::Ease;
use tracing::info;

use crate::PlayingState;

use super::{terra::Plane, terrain_spawner::FilledLot};

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(PlayingState::SwitchingPlane).with_system(change_plane),
        )
        .add_system_set(SystemSet::on_update(PlayingState::SwitchingPlane).with_system(tick))
        .add_system_set(SystemSet::on_exit(PlayingState::SwitchingPlane).with_system(clear));
    }
}

struct SwitchingTimer(Timer);

fn change_plane(mut commands: Commands, mut plane: ResMut<Plane>) {
    match *plane {
        Plane::Material => {
            *plane = Plane::Ethereal;
        }
        Plane::Ethereal => {
            *plane = Plane::Material;
        }
    }
    commands.insert_resource(SwitchingTimer(Timer::from_seconds(1.0, false)));
    info!("now on {:?} plane", *plane);
}

fn tick(
    mut lots: Query<(&mut Transform, &FilledLot)>,
    time: Res<Time>,
    mut timer: ResMut<SwitchingTimer>,
    mut playing_state: ResMut<State<PlayingState>>,
    plane: Res<Plane>,
    mut light: Query<&mut DirectionalLight>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        playing_state.set(PlayingState::Playing).unwrap();
    }
    let percent = timer.0.percent().calc(EaseFunction::CubicInOut);
    for (mut transform, lot) in &mut lots {
        transform.rotation = match (lot.plane == *plane, (lot.x + lot.z) % 2 == 0) {
            (true, true) => Quat::from_axis_angle(Vec3::Z, PI * percent + PI),
            (true, false) => Quat::from_axis_angle(Vec3::X, PI * percent + PI),
            (false, true) => Quat::from_axis_angle(Vec3::Z, PI * percent),
            (false, false) => Quat::from_axis_angle(Vec3::X, PI * percent),
        };
    }

    light.single_mut().color = match *plane {
        Plane::Material => {
            EaseValue(Color::CYAN)
                .lerp(&EaseValue(Color::WHITE), &timer.0.percent())
                .0
        }
        Plane::Ethereal => {
            EaseValue(Color::WHITE)
                .lerp(&EaseValue(Color::CYAN), &timer.0.percent())
                .0
        }
    }
}

fn clear(mut commands: Commands, lots: Query<(Entity, &FilledLot)>, plane: Res<Plane>) {
    for (entity, lot) in &lots {
        if lot.plane != *plane {
            commands.entity(entity).despawn_recursive();
        }
    }
}
