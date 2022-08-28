use std::f32::consts::PI;

use bevy::{
    prelude::{
        App, Color, Commands, DespawnRecursiveExt, DirectionalLight, Entity, Or, Quat, Query, Res,
        ResMut, State, SystemSet, Transform, Vec3, Visibility, With,
    },
    time::{Time, Timer},
};
use bevy_easings::{EaseFunction, EaseValue, Lerp};
use interpolation::Ease;
use tracing::info;

use super::{
    terra::Plane,
    terrain_spawner::FilledLot,
    zombies::{IdleZombie, Zombie},
    PlayingState,
};

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

fn change_plane(
    mut commands: Commands,
    mut plane: ResMut<Plane>,
    mut zombies: Query<&mut Visibility, Or<(With<Zombie>, With<IdleZombie>)>>,
) {
    match *plane {
        Plane::Material => {
            *plane = Plane::Ethereal;
        }
        Plane::Ethereal => {
            *plane = Plane::Material;
        }
    }
    commands.insert_resource(SwitchingTimer(Timer::from_seconds(1.0, false)));
    for mut visibility in &mut zombies {
        if visibility.is_visible == true {
            visibility.is_visible = false;
        }
    }
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

    let material = Color::WHITE;
    let ethereal = Color::rgb(1.0, 0.7, 1.0);
    light.single_mut().color = match *plane {
        Plane::Material => {
            EaseValue(ethereal)
                .lerp(&EaseValue(material), &timer.0.percent())
                .0
        }
        Plane::Ethereal => {
            EaseValue(material)
                .lerp(&EaseValue(ethereal), &timer.0.percent())
                .0
        }
    }
}

fn clear(
    mut commands: Commands,
    lots: Query<(Entity, &FilledLot)>,
    plane: Res<Plane>,
    mut zombies: Query<
        (&mut Visibility, Option<&Zombie>, Option<&IdleZombie>),
        Or<(With<Zombie>, With<IdleZombie>)>,
    >,
) {
    for (entity, lot) in &lots {
        if lot.plane != *plane {
            commands.entity(entity).despawn_recursive();
        }
    }
    for (mut visibility, zombie, idle) in &mut zombies {
        if zombie.map(|z| z.plane == *plane).unwrap_or_default() {
            visibility.is_visible = true;
        }
        if idle.map(|z| z.plane == *plane).unwrap_or_default() {
            visibility.is_visible = true;
        }
    }
}
