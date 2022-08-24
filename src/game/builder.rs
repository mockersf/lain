use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        shape, AlphaMode, App, Assets, Color, Commands, Component, DespawnRecursiveExt, Entity,
        FromWorld, Handle, Mesh, PbrBundle, Query, Res, ResMut, StandardMaterial, SystemSet,
        Transform, Vec3, With,
    },
    utils::default,
};

use super::{
    heightmap::LOW_DEF,
    terra::Plane,
    terrain_spawner::{CursorPosition, Map},
    PlayingState,
};

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorMaterials>()
            .add_system_set(SystemSet::on_enter(PlayingState::Building).with_system(display_cursor))
            .add_system_set(SystemSet::on_exit(PlayingState::Building).with_system(clear))
            .add_system_set(
                SystemSet::on_update(PlayingState::Building).with_system(update_cursor),
            );
    }
}

struct CursorMaterials {
    valid: Handle<StandardMaterial>,
    invalid: Handle<StandardMaterial>,
}

impl FromWorld for CursorMaterials {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        CursorMaterials {
            valid: materials.add(StandardMaterial {
                base_color: Color::rgba(0.2, 1.0, 0.2, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            invalid: materials.add(StandardMaterial {
                base_color: Color::rgba(1.0, 0.2, 0.2, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
        }
    }
}

fn display_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<CursorMaterials>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(
                1.0 / LOW_DEF as f32,
                0.75,
                1.0 / LOW_DEF as f32,
            ))),
            material: materials.valid.clone_weak(),
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
    mut cursor: Query<(&mut Transform, &mut Handle<StandardMaterial>), With<CursorSelection>>,
    map: Res<Map>,
    plane: Res<Plane>,
    materials: Res<CursorMaterials>,
) {
    let (mut transform, mut material) = cursor.single_mut();
    transform.translation = cursor_position.world;
    if map
        .lots
        .get(&(cursor_position.map, *plane))
        .and_then(|lot| lot.get(&cursor_position.lot))
        .map(|o| o.is_free())
        .unwrap_or(true)
    {
        if *material == materials.invalid {
            *material = materials.valid.clone_weak();
        }
    } else {
        if *material == materials.valid {
            *material = materials.invalid.clone_weak();
        }
    }
}
