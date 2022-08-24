use std::f32::consts::PI;

use bevy::{
    ecs::component::SparseStorage,
    pbr::NotShadowCaster,
    prelude::*,
    tasks::AsyncComputeTaskPool,
    utils::{Entry, HashMap},
};
use bevy_mod_raycast::{Intersection, RayCastMesh, RayCastMethod, RayCastSource, SimplifiedMesh};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    assets::BuildingAssets,
    game::heightmap::{HeightMap, LOW_DEF},
    game::terra::{Plane, TerraNoises},
    GameState, PlayingState,
};

const BORDER: f32 = 20.0;

#[cfg(target = "wasm32-unknown-unknown")]
const CHANNEL_SIZE: usize = 1;
#[cfg(not(target = "wasm32-unknown-unknown"))]
const CHANNEL_SIZE: usize = 20;

#[derive(Debug)]
pub(crate) struct EmptyLot {
    x: i32,
    z: i32,
    offscreen: bool,
    loading: bool,
}

impl Component for EmptyLot {
    type Storage = SparseStorage;
}

impl EmptyLot {
    pub(crate) fn new(position: IVec2, offscreen: bool) -> Self {
        EmptyLot {
            x: position.x,
            z: position.y,
            offscreen,
            loading: false,
        }
    }
}

pub(crate) struct FilledLot {
    pub(crate) x: i32,
    pub(crate) z: i32,
    pub(crate) plane: Plane,
}

impl Component for FilledLot {
    type Storage = SparseStorage;
}

pub(crate) struct TerrainSpawnerPlugin;

struct MyChannel(Sender<InTransitLot>, Receiver<InTransitLot>);

impl Plugin for TerrainSpawnerPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::bounded(CHANNEL_SIZE);

        let mut map = BuildingMap::default();
        map.lots.insert(
            (IVec2::new(0, 0), Plane::Material),
            vec![(
                IVec2::new(LOW_DEF as i32 / 2, LOW_DEF as i32 / 2),
                BuildingType::Crystal,
            )],
        );
        map.lots.insert(
            (IVec2::new(0, 0), Plane::Ethereal),
            vec![(
                IVec2::new(LOW_DEF as i32 / 2, LOW_DEF as i32 / 2),
                BuildingType::Crystal,
            )],
        );

        app.insert_resource(MyChannel(tx, rx))
            .init_resource::<VisibleLots>()
            .insert_resource(map)
            .insert_resource(Plane::Material)
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_camera))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(move_camera)
                    .with_system(fill_empty_lots)
                    .with_system(refresh_visible_lots.after(fill_empty_lots))
                    .with_system(cleanup_lots)
                    .with_system(intersection),
            );
    }
}

fn setup_camera(
    mut commands: Commands,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut transform = camera.single_mut();
    *transform = Transform::from_xyz(0.0, 4.0, -0.5).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
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

struct InTransitLot {
    mesh: Mesh,
    material_color: Image,
    ethereal_color: Image,
    metallic_roughness: Image,
    x: i32,
    z: i32,
}
struct HandledLot {
    mesh: Handle<Mesh>,
    color: Handle<StandardMaterial>,
}

#[derive(Debug)]
enum BuildingType {
    Crystal,
}

#[derive(Default)]
struct BuildingMap {
    lots: HashMap<(IVec2, Plane), Vec<(IVec2, BuildingType)>>,
}

#[allow(clippy::type_complexity)]
fn fill_empty_lots(
    mut commands: Commands,
    mut lots: Query<(Entity, &mut EmptyLot, &mut Transform)>,
    (mut meshes, mut textures, mut materials): (
        ResMut<Assets<Mesh>>,
        ResMut<Assets<Image>>,
        ResMut<Assets<StandardMaterial>>,
    ),
    mut mesh_cache: Local<HashMap<(IVec2, Plane), HandledLot>>,
    noises: Res<TerraNoises>,
    channel: Res<MyChannel>,
    mut in_transit: Local<usize>,
    plane: Res<Plane>,
    playing_state: Res<State<PlayingState>>,
    map: Res<BuildingMap>,
    building_assets: Res<BuildingAssets>,
) {
    for (entity, mut position, mut transform) in lots.iter_mut() {
        if let Some(mesh) = mesh_cache.get(&(IVec2::new(position.x, position.z), *plane)) {
            if !position.offscreen {
                commands
                    .entity(entity)
                    .with_children(|lot| {
                        lot.spawn_bundle(PbrBundle {
                            mesh: mesh.mesh.clone_weak(),
                            material: mesh.color.clone_weak(),
                            transform: Transform::from_xyz(0.0, 0.035, 0.0),
                            ..default()
                        })
                        .insert_bundle((
                            RayCastMesh::<RaycastSet>::default(),
                            SimplifiedMesh {
                                mesh: meshes.add(Mesh::from(shape::Plane::default())),
                            },
                        ));

                        if let Some(building_lot) =
                            map.lots.get(&(IVec2::new(position.x, position.z), *plane))
                        {
                            for building in building_lot {
                                match building.1 {
                                    BuildingType::Crystal => lot.spawn_bundle(SceneBundle {
                                        scene: building_assets.crystal.clone_weak(),
                                        transform: Transform {
                                            scale: Vec3::splat(1.0 / LOW_DEF as f32),
                                            translation: Vec3::new(
                                                (building.0.x - LOW_DEF as i32 / 2) as f32
                                                    / LOW_DEF as f32,
                                                0.03,
                                                (building.0.y - LOW_DEF as i32 / 2) as f32
                                                    / LOW_DEF as f32,
                                            ),
                                            ..default()
                                        },
                                        ..default()
                                    }),
                                };
                            }
                        }
                    })
                    .insert(FilledLot {
                        x: position.x,
                        z: position.z,
                        plane: *plane,
                    })
                    .remove::<EmptyLot>();
                if *playing_state.current() == PlayingState::SwitchingPlane {
                    transform.rotation = Quat::from_axis_angle(Vec3::X, PI);
                }
            } else {
                commands.entity(entity).remove::<EmptyLot>();
            }
        } else if !position.loading && *in_transit < CHANNEL_SIZE {
            let pos_x = position.x as f32;
            let pos_y = position.z as f32;
            let noises = *noises;
            let tx = channel.0.clone();
            let plane = *plane;
            AsyncComputeTaskPool::get()
                .spawn(async move {
                    let heightmap = HeightMap::build_heightmap(pos_x, pos_y, plane, noises);
                    let terrain = heightmap.into_mesh_and_texture();

                    tx.send(InTransitLot {
                        mesh: terrain.mesh,
                        material_color: terrain.material_color,
                        ethereal_color: terrain.ethereal_color,
                        metallic_roughness: terrain.metallic_roughness,
                        x: pos_x as i32,
                        z: pos_y as i32,
                    })
                    .unwrap();
                })
                .detach();
            position.loading = true;
            *in_transit += 1;
        }
        for lot in channel.1.try_iter() {
            let mesh_handle = meshes.add(lot.mesh);
            let mr_texture = textures.add(lot.metallic_roughness);
            let material_handled_lot = HandledLot {
                mesh: mesh_handle.clone(),
                color: materials.add(StandardMaterial {
                    base_color: bevy::render::color::Color::WHITE,
                    base_color_texture: Some(textures.add(lot.material_color)),
                    perceptual_roughness: 1.0,
                    metallic: 1.0,
                    metallic_roughness_texture: Some(mr_texture.clone()),
                    ..Default::default()
                }),
            };
            let ethereal_handled_lot = HandledLot {
                mesh: mesh_handle,
                color: materials.add(StandardMaterial {
                    base_color: bevy::render::color::Color::WHITE,
                    base_color_texture: Some(textures.add(lot.ethereal_color)),
                    perceptual_roughness: 1.0,
                    metallic: 1.0,
                    metallic_roughness_texture: Some(mr_texture),
                    ..Default::default()
                }),
            };
            *in_transit -= 1;
            mesh_cache.insert(
                (IVec2::new(lot.x, lot.z), Plane::Material),
                material_handled_lot,
            );
            mesh_cache.insert(
                (IVec2::new(lot.x, lot.z), Plane::Ethereal),
                ethereal_handled_lot,
            );
        }
    }
}

#[allow(clippy::type_complexity)]
fn cleanup_lots(
    mut commands: Commands,
    lots: Query<Entity, (Without<EmptyLot>, Without<Transform>, Without<Children>)>,
) {
    for entity in lots.iter() {
        commands.entity(entity).despawn();
    }
}

#[derive(Default)]
pub(crate) struct VisibleLots(HashMap<IVec2, (Entity, Plane)>);

fn refresh_visible_lots(
    mut commands: Commands,
    camera: Query<(&bevy::render::camera::Camera, &GlobalTransform)>,
    mut visible_lots: ResMut<VisibleLots>,
    plane: Res<Plane>,
    playing_state: Res<State<PlayingState>>,
) {
    let margin = 0.5;
    let is_on_screen = |position: Vec3| {
        if position.x - margin > 1.0 {
            return false;
        }
        if position.x + margin < -1.0 {
            return false;
        }
        if position.y - margin > 1.0 {
            return false;
        }
        if position.y + margin < -1.0 {
            return false;
        }

        true
    };

    let (camera, gt) = camera.single();

    let mut updated_lots: HashMap<IVec2, (Entity, Plane)> = visible_lots
        .0
        .drain()
        .filter(|(position, (entity, lot_plane))| {
            if let Some(screen_position) =
                camera.world_to_ndc(gt, Vec3::new(position.x as f32, 0.0, position.y as f32))
            {
                if !is_on_screen(screen_position) || *plane != *lot_plane {
                    if *playing_state.current() != PlayingState::SwitchingPlane {
                        info!("despawning {} / {}", position.x, position.y);
                        commands.entity(*entity).despawn_recursive();
                    }
                    return false;
                }
            }
            true
        })
        .collect();

    let span = gt.translation().y as i32 + 1;
    for i in -span..span {
        for j in -(span / 2)..span {
            let position = IVec2::new(gt.translation().x as i32 + i, gt.translation().z as i32 + j);
            if let Some(screen_position) =
                camera.world_to_ndc(gt, Vec3::new(position.x as f32, 0.0, position.y as f32))
            {
                if is_on_screen(screen_position) {
                    if let Entry::Vacant(vacant) = updated_lots.entry(position) {
                        info!("spawning {} / {}", position.x, position.y);
                        vacant.insert((
                            commands
                                .spawn_bundle((
                                    EmptyLot::new(position, false),
                                    Transform::from_xyz(position.x as f32, 0.0, position.y as f32),
                                    GlobalTransform::identity(),
                                    Visibility::visible(),
                                    ComputedVisibility::not_visible(),
                                ))
                                .id(),
                            *plane,
                        ));
                    }
                }
            }
        }
    }

    visible_lots.0 = updated_lots;
}

fn move_camera(
    mut query: Query<&mut Transform, With<Camera>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut playing_state: ResMut<State<PlayingState>>,
) {
    if *playing_state.current() != PlayingState::SwitchingPlane {
        let transform = query.single();
        let move_by = time.delta_seconds();
        let mut move_to = Vec3::ZERO;
        let mut moving = false;
        if input.pressed(KeyCode::Left) && transform.translation.x < BORDER {
            moving = true;
            move_to.x = 1.0;
        } else if input.pressed(KeyCode::Right) && transform.translation.x > -BORDER {
            moving = true;
            move_to.x = -1.0;
        }
        if input.pressed(KeyCode::Up) && transform.translation.z < BORDER {
            moving = true;
            move_to.z = 1.0;
        } else if input.pressed(KeyCode::Down) && transform.translation.z > -BORDER {
            moving = true;
            move_to.z = -1.0;
        }
        if input.pressed(KeyCode::A) && transform.translation.y < 20.0 {
            moving = true;
            move_to.y += 0.2;
        } else if input.pressed(KeyCode::Q) && transform.translation.y > 2.0 {
            moving = true;
            move_to.y -= 0.2;
        }
        if moving {
            query.single_mut().translation += move_to.normalize() * move_by;
        }
        if input.just_pressed(KeyCode::Space) {
            playing_state.set(PlayingState::SwitchingPlane).unwrap();
        }
    }
}

pub(crate) struct RaycastSet;

fn world_to_map(world: Vec2) -> (IVec2, IVec2) {
    (
        IVec2::new(world.x.round() as i32, world.y.round() as i32),
        IVec2::new(
            ((1.0 - (world.x - world.x.round() + 0.5)) * LOW_DEF as f32) as i32,
            ((world.y - world.y.round() + 0.5) * LOW_DEF as f32) as i32,
        ),
    )
}

fn map_to_world(map: (IVec2, IVec2)) -> Vec2 {
    Vec2::new(
        map.0.x as f32 - (map.1.x as f32 + 0.5) / LOW_DEF as f32 + 0.5,
        map.0.y as f32 + (map.1.y as f32 + 0.5) / LOW_DEF as f32 - 0.5,
    )
}

fn intersection(
    query: Query<&Intersection<RaycastSet>>,
    mut cursor: EventReader<CursorMoved>,
    mut pick_source: Query<&mut RayCastSource<RaycastSet>>,
    mut cursor_position: Query<&mut Transform, With<CursorSelection>>,
) {
    for intersection in &query {
        if let Some(position) = intersection.position() {
            let position = map_to_world(world_to_map(Vec2::new(position.x, position.z)));
            cursor_position.single_mut().translation = Vec3::new(position.x, 0.05, position.y);
        }
    }
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    pick_source.single_mut().cast_method = RayCastMethod::Screenspace(cursor_position);
}