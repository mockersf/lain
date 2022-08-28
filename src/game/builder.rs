use bevy::{
    pbr::NotShadowCaster,
    prelude::{
        shape, AlphaMode, App, Assets, BuildChildren, Color, Commands, Component,
        DespawnRecursiveExt, Entity, FromWorld, Handle, Input, Mesh, MouseButton, PbrBundle, Query,
        Res, ResMut, StandardMaterial, SystemSet, Transform, Vec2, Vec3, With,
    },
    scene::SceneBundle,
    time::Timer,
    utils::default,
    window::Windows,
};

use crate::{assets::BuildingAssets, game::terrain_spawner::map_to_world};

use super::{
    heightmap::LOW_DEF,
    nests::ZombieNest,
    stats::GameTag,
    terra::Plane,
    terrain_spawner::{CursorPosition, FilledLot, Map, Occupying, Pathfinding, TOWER_SCALE},
    towers::Tower,
    PlayingState,
};

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorMaterials>()
            .add_system_set(SystemSet::on_enter(PlayingState::Building).with_system(display_cursor))
            .add_system_set(SystemSet::on_exit(PlayingState::Building).with_system(clear))
            .add_system_set(
                SystemSet::on_update(PlayingState::Building)
                    .with_system(update_cursor)
                    .with_system(build),
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
    } else if *material == materials.valid {
        *material = materials.invalid.clone_weak();
    }
}

fn build(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut map: ResMut<Map>,
    cursor_position: Res<CursorPosition>,
    plane: Res<Plane>,
    mut commands: Commands,
    lots: Query<(Entity, &FilledLot)>,
    building_assets: Res<BuildingAssets>,
    cursor: Query<&Handle<StandardMaterial>, With<CursorSelection>>,
    materials: Res<CursorMaterials>,
    pathfinding: Res<Pathfinding>,
    nests: Query<&ZombieNest>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        if let Some(pos) = windows.primary().cursor_position() {
            if pos.x < 140.0 && pos.y > 550.0 {
                // in UI zone
                return;
            }
        } else {
            // outside
            return;
        }

        if *cursor.single() == materials.valid {
            let mut temp_mesh = pathfinding.clone();
            temp_mesh.cut_polygon_out((cursor_position.map, cursor_position.lot));
            for nest in nests.iter() {
                let position = map_to_world((nest.map, nest.lot));
                if !temp_mesh.mesh.path(position, Vec2::ZERO).complete {
                    return;
                }
            }
            map.lots
                .get_mut(&(cursor_position.map, *plane))
                .unwrap()
                .insert(cursor_position.lot, Occupying::Tower);
            map.lots
                .get_mut(&(cursor_position.map, plane.next()))
                .unwrap()
                .insert(cursor_position.lot, Occupying::Block);
            for (entity, lot) in &lots {
                if lot.x == cursor_position.map.x && lot.z == cursor_position.map.y {
                    commands.entity(entity).add_children(|lot| {
                        lot.spawn_bundle(SceneBundle {
                            scene: if *plane == Plane::Material {
                                building_assets.material_tower.clone_weak()
                            } else {
                                building_assets.ethereal_tower.clone_weak()
                            },
                            transform: Transform {
                                scale: Vec3::splat(TOWER_SCALE / LOW_DEF as f32),
                                translation: Vec3::new(
                                    -(cursor_position.lot.x - LOW_DEF as i32 / 2) as f32
                                        / LOW_DEF as f32,
                                    0.03,
                                    (cursor_position.lot.y - LOW_DEF as i32 / 2) as f32
                                        / LOW_DEF as f32,
                                ),
                                ..default()
                            },
                            ..default()
                        });
                    })
                }
            }
            commands.spawn_bundle((
                Tower {
                    timer: Timer::from_seconds(1.0, true),
                    strength: 1.0,
                    plane: *plane,
                },
                Transform::from_translation(cursor_position.world),
                GameTag,
            ));
        }
    }
}
