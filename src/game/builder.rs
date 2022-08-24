use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        shape, AlphaMode, App, Assets, Color, Commands, Component, DespawnRecursiveExt, Entity,
        Mesh, PbrBundle, Query, Res, ResMut, StandardMaterial, SystemSet, Transform, Vec3, With,
    },
    utils::default,
};

use super::{heightmap::LOW_DEF, terrain_spawner::CursorPosition, PlayingState};

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(PlayingState::Building).with_system(display_cursor))
            .add_system_set(SystemSet::on_exit(PlayingState::Building).with_system(clear))
            .add_system_set(
                SystemSet::on_update(PlayingState::Building).with_system(update_cursor),
            );
    }
}

fn display_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                1.0 / LOW_DEF as f32,
                0.7,
                1.0 / LOW_DEF as f32,
            ))),
            material: materials.add(StandardMaterial {
                base_color: Color::rgba(0.2, 1.0, 0.2, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert_bundle((CursorSelection, NotShadowCaster));
}

#[derive(Component)]
struct CursorSelection;

fn clear(mut commands: Commands, cursor: Query<Entity, With<CursorSelection>>) {
    commands.entity(cursor.single()).despawn_recursive();
}

fn update_cursor(
    cursor_position: Res<CursorPosition>,
    mut cursor: Query<&mut Transform, With<CursorSelection>>,
) {
    cursor.single_mut().translation = cursor_position.world;
}
