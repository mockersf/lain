use std::f32::consts::PI;

use bevy::{
    ecs::component::SparseStorage,
    prelude::*,
    tasks::AsyncComputeTaskPool,
    utils::{Entry, HashMap},
};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    terra::{Plane, TerraNoises},
    GameState, PlayingState,
};

const BORDER: f32 = 20.0;

#[cfg(target = "wasm32-unknown-unknown")]
const CHANNEL_SIZE: usize = 1;
#[cfg(not(target = "wasm32-unknown-unknown"))]
const CHANNEL_SIZE: usize = 20;

#[derive(Debug)]
pub struct EmptyLot {
    x: i32,
    z: i32,
    offscreen: bool,
    loading: bool,
}

impl Component for EmptyLot {
    type Storage = SparseStorage;
}

impl EmptyLot {
    pub fn new(position: IVec2, offscreen: bool) -> Self {
        EmptyLot {
            x: position.x,
            z: position.y,
            offscreen,
            loading: false,
        }
    }
}

pub struct FilledLot {
    x: i32,
    z: i32,
    plane: Plane,
}

impl Component for FilledLot {
    type Storage = SparseStorage;
}

pub struct TerrainSpawnerPlugin;

struct MyChannel(Sender<InTransitLot>, Receiver<InTransitLot>);

impl Plugin for TerrainSpawnerPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::bounded(CHANNEL_SIZE);

        app.insert_resource(MyChannel(tx, rx))
            .init_resource::<VisibleLots>()
            .insert_resource(Plane::Material)
            .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup_camera))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(move_camera)
                    .with_system(fill_empty_lots)
                    .with_system(cleanup_lots)
                    .with_system(refresh_visible_lots.after(fill_empty_lots))
                    .with_system(cleanup_lots),
            );
    }
}

fn setup_camera(mut camera: Query<&mut Transform, With<Camera>>) {
    let mut transform = camera.single_mut();
    *transform = Transform::from_xyz(0.0, 4.0, -0.5).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
}

struct InTransitLot {
    mesh: Mesh,
    color: Image,
    metallic_roughness: Image,
    x: i32,
    z: i32,
    plane: Plane,
}
struct HandledLot {
    mesh: Handle<Mesh>,
    color: Handle<StandardMaterial>,
}

#[allow(clippy::type_complexity)]
fn fill_empty_lots(
    mut commands: Commands,
    mut lots: Query<(Entity, &mut EmptyLot)>,
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
) {
    for (entity, mut position) in lots.iter_mut() {
        if let Some(mesh) = mesh_cache.get(&(IVec2::new(position.x, position.z), *plane)) {
            if !position.offscreen {
                commands
                    .entity(entity)
                    .with_children(|lot| {
                        lot.spawn_bundle(PbrBundle {
                            mesh: mesh.mesh.clone_weak(),
                            material: mesh.color.clone_weak(),
                            transform: Transform::from_xyz(0.0, 0.1, 0.0),
                            ..default()
                        });
                    })
                    .insert(FilledLot {
                        x: position.x,
                        z: position.z,
                        plane: *plane,
                    })
                    .remove::<EmptyLot>();
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
                    let heightmap =
                        crate::heightmap::HeightMap::build_heightmap(pos_x, pos_y, plane, noises);
                    let terrain = heightmap.into_mesh_and_texture();

                    tx.send(InTransitLot {
                        mesh: terrain.mesh,
                        color: terrain.color,
                        metallic_roughness: terrain.metallic_roughness,
                        x: pos_x as i32,
                        z: pos_y as i32,
                        plane,
                    })
                    .unwrap();
                })
                .detach();
            position.loading = true;
            *in_transit += 1;
        }
        for lot in channel.1.try_iter() {
            let handled_lot = HandledLot {
                mesh: meshes.add(lot.mesh),
                color: materials.add(StandardMaterial {
                    base_color: bevy::render::color::Color::WHITE,
                    base_color_texture: Some(textures.add(lot.color)),
                    perceptual_roughness: 1.0,
                    metallic: 1.0,
                    metallic_roughness_texture: Some(textures.add(lot.metallic_roughness)),
                    ..Default::default()
                }),
            };
            *in_transit -= 1;
            mesh_cache.insert((IVec2::new(lot.x, lot.z), lot.plane), handled_lot);
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
pub struct VisibleLots(HashMap<IVec2, (Entity, Plane)>);

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
                        commands.entity(*entity).despawn_recursive();
                    }
                    return false;
                }
            }
            true
        })
        .collect();

    let span = 5;
    for i in -span..span {
        for j in -(span / 2)..span {
            let position = IVec2::new(gt.translation().x as i32 + i, gt.translation().z as i32 + j);
            if let Some(screen_position) =
                camera.world_to_ndc(gt, Vec3::new(position.x as f32, 0.0, position.y as f32))
            {
                if is_on_screen(screen_position) {
                    if let Entry::Vacant(vacant) = updated_lots.entry(position) {
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
        if moving {
            query.single_mut().translation += move_to.normalize() * move_by;
        }
        if input.just_pressed(KeyCode::Space) {
            playing_state.set(PlayingState::SwitchingPlane).unwrap();
        }
    }
}

pub struct SwitchingPlanePlugin;
impl Plugin for SwitchingPlanePlugin {
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
    mut light: Query<&mut DirectionalLight>,
) {
    match *plane {
        Plane::Material => {
            *plane = Plane::Ethereal;
            light.single_mut().color = Color::ALICE_BLUE;
        }
        Plane::Ethereal => {
            *plane = Plane::Material;
            light.single_mut().color = Color::WHITE;
        }
    }
    commands.insert_resource(SwitchingTimer(Timer::from_seconds(1.0, false)));
    info!("now on plane {:?}", *plane);
}

fn tick(
    mut lots: Query<(&mut Transform, &FilledLot)>,
    time: Res<Time>,
    mut timer: ResMut<SwitchingTimer>,
    mut playing_state: ResMut<State<PlayingState>>,
    plane: Res<Plane>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        playing_state.set(PlayingState::Playing).unwrap();
    }
    for (mut transform, lot) in &mut lots {
        transform.rotation = match (lot.plane == *plane, (lot.x + lot.z) % 2 == 0) {
            (true, true) => Quat::from_axis_angle(Vec3::Z, PI * timer.0.percent() + PI),
            (true, false) => Quat::from_axis_angle(Vec3::X, PI * timer.0.percent() + PI),
            (false, true) => Quat::from_axis_angle(Vec3::Z, PI * timer.0.percent()),
            (false, false) => Quat::from_axis_angle(Vec3::X, PI * timer.0.percent()),
        };
    }
}

fn clear(mut commands: Commands, lots: Query<(Entity, &FilledLot)>, plane: Res<Plane>) {
    for (entity, lot) in &lots {
        if lot.plane != *plane {
            commands.entity(entity).despawn_recursive();
        }
    }
}
