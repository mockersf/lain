use bevy::{
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            Indices,
        },
        render_resource::{PipelineCache, PrimitiveTopology},
        RenderApp, RenderStage,
    },
};
use crossbeam_channel::Receiver;
use rand::Rng;

use crate::{assets::AllTheLoading, ui_helper::ColorScheme};

const CURRENT_STATE: crate::GameState = crate::GameState::Splash;

#[derive(Component)]
struct ScreenTag;

struct Screen {
    done: Timer,
    rx: Receiver<bool>,
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::bounded(1);

        app.insert_resource(Screen {
            done: Timer::from_seconds(1.0, false),
            rx,
        })
        .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
        .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
        .add_system_set(
            SystemSet::on_update(CURRENT_STATE)
                .with_system(done)
                .with_system(animate_logo)
                .with_system(pipeline_preloader),
        );

        let renderer_app = app.sub_app_mut(RenderApp);
        let mut done = false;
        renderer_app.add_system_to_stage(RenderStage::Cleanup, move |cache: Res<PipelineCache>| {
            if !done && cache.ready() >= 10 {
                let _ = tx.send(true);
                done = true
            }
        });
    }
}

fn pipeline_preloader(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skinned_mesh_inverse_bindposes_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut loading_state: ResMut<State<AllTheLoading>>,
    mut loaded: Local<u32>,
    mut status: Query<&mut Text>,
    screen: Res<Screen>,
) {
    if *loading_state.current() == AllTheLoading::Pipelines {
        if *loaded == 0 {
            status.single_mut().sections[0].value = "Loading Pipelines...".to_string();
            *loaded += 1;
        } else if *loaded == 1 {
            {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(shape::Cube::new(0.1).into()),
                        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        ..default()
                    })
                    .insert(ScreenTag);
            }

            {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(shape::Cube::new(0.1).into()),
                        material: materials.add(StandardMaterial {
                            base_color: Color::BLUE,
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        }),
                        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        ..default()
                    })
                    .insert(ScreenTag);
            }

            {
                let inverse_bindposes = skinned_mesh_inverse_bindposes_assets.add(
                    SkinnedMeshInverseBindposes::from(vec![
                        Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)),
                        Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)),
                    ]),
                );
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![[0.0, 1.0, 0.0], [0.1, 1.0, 0.0], [0.2, 1.1, 0.0]],
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_JOINT_INDEX,
                    vec![[0u16, 0, 0, 0], [0, 0, 0, 0], [0, 1, 0, 0]],
                );
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_JOINT_WEIGHT,
                    vec![
                        [1.00, 0.00, 0.0, 0.0],
                        [1.00, 0.00, 0.0, 0.0],
                        [0.75, 0.25, 0.0, 0.0],
                    ],
                );
                mesh.set_indices(Some(Indices::U16(vec![0, 1, 2])));

                let mesh = meshes.add(mesh);
                let joint_0 = commands
                    .spawn_bundle((
                        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        GlobalTransform::default(),
                        ScreenTag,
                    ))
                    .id();
                let joint_1 = commands
                    .spawn_bundle((
                        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        GlobalTransform::default(),
                        ScreenTag,
                    ))
                    .id();
                commands.entity(joint_0).push_children(&[joint_1]);

                let joint_entities = vec![joint_0, joint_1];
                commands
                    .spawn_bundle(PbrBundle {
                        mesh,
                        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        ..default()
                    })
                    .insert(SkinnedMesh {
                        inverse_bindposes,
                        joints: joint_entities,
                    })
                    .insert(ScreenTag);
            }

            {
                let inverse_bindposes = skinned_mesh_inverse_bindposes_assets.add(
                    SkinnedMeshInverseBindposes::from(vec![
                        Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)),
                        Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)),
                    ]),
                );
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![[0.0, 1.0, 0.0], [0.1, 1.0, 0.0], [0.2, 1.1, 0.0]],
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 1.0, 1.0]; 3]);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; 3]);
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_JOINT_INDEX,
                    vec![[0u16, 0, 0, 0], [0, 0, 0, 0], [0, 1, 0, 0]],
                );
                mesh.insert_attribute(
                    Mesh::ATTRIBUTE_JOINT_WEIGHT,
                    vec![
                        [1.00, 0.00, 0.0, 0.0],
                        [1.00, 0.00, 0.0, 0.0],
                        [0.75, 0.25, 0.0, 0.0],
                    ],
                );
                mesh.set_indices(Some(Indices::U16(vec![0, 1, 2])));

                let mesh = meshes.add(mesh);
                let joint_0 = commands
                    .spawn_bundle((
                        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        GlobalTransform::default(),
                        ScreenTag,
                    ))
                    .id();
                let joint_1 = commands
                    .spawn_bundle((
                        Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        GlobalTransform::default(),
                        ScreenTag,
                    ))
                    .id();
                commands.entity(joint_0).push_children(&[joint_1]);

                let joint_entities = vec![joint_0, joint_1];
                commands
                    .spawn_bundle(PbrBundle {
                        mesh,
                        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                        material: materials.add(StandardMaterial {
                            cull_mode: None,
                            ..default()
                        }),
                        ..default()
                    })
                    .insert(SkinnedMesh {
                        inverse_bindposes,
                        joints: joint_entities,
                    })
                    .insert(ScreenTag);
            }
            *loaded += 1;
        } else if *loaded == 2 {
            if screen.rx.try_recv().unwrap_or_default() {
                let _ = loading_state.set(AllTheLoading::Done);
                status.single_mut().sections[0].value = "Ready!".to_string();
            }
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading screen");

    let vleue_logo = asset_server.load("branding/logo.png");
    let bevy_logo = asset_server.load("branding/bevy_logo_dark.png");
    let birdoggo_logo = asset_server.load("branding/birdoggo.png");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Auto),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },

                    image: UiImage(vleue_logo),
                    ..default()
                })
                .insert(SplashGiggle(Timer::from_seconds(0.05, true)));
            commands.spawn_bundle(TextBundle {
                style: Style {
                    position: UiRect {
                        left: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    ..default()
                },
                text: Text::from_section(
                    "Loading Assets...",
                    TextStyle {
                        font: asset_server.load("fonts/mandrill.ttf"),
                        font_size: 20.0,
                        color: ColorScheme::TEXT_DARK,
                    },
                ),
                ..default()
            });
            commands.spawn_bundle(ImageBundle {
                style: Style {
                    position: UiRect {
                        right: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    ..default()
                },
                image: UiImage(bevy_logo),
                ..default()
            });
            commands.spawn_bundle(ImageBundle {
                style: Style {
                    position: UiRect {
                        right: Val::Px(10.0),
                        bottom: Val::Px(70.0),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    ..default()
                },
                image: UiImage(birdoggo_logo),
                ..default()
            });
        })
        .insert(ScreenTag);

    let size = 5.0;
    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -std::f32::consts::FRAC_PI_4),
            ..default()
        },
        directional_light: DirectionalLight {
            shadows_enabled: true,
            shadow_projection: OrthographicProjection {
                left: -size,
                right: size,
                bottom: -size,
                top: size,
                near: -size,
                far: size,
                ..Default::default()
            },
            illuminance: 20000.0,
            ..default()
        },
        ..default()
    });
}

#[derive(Component)]
struct SplashGiggle(Timer);

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn done(
    time: Res<Time>,
    mut screen: ResMut<Screen>,
    mut state: ResMut<State<crate::GameState>>,
    loading_state: Res<State<AllTheLoading>>,
) {
    if screen.done.tick(time.delta()).finished() && loading_state.current() == &AllTheLoading::Done
    {
        state.set(crate::GameState::Menu).unwrap();
    }
}

fn animate_logo(time: Res<Time>, mut query: Query<(&mut SplashGiggle, &mut Transform)>) {
    for (mut timer, mut transform) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            let scale = transform.scale;
            if (scale.x - 1.) > 0.01 {
                *transform = Transform::identity();
                continue;
            }

            let mut rng = rand::thread_rng();
            let act = rng.gen_range(0..100);
            if act > 50 {
                let scale_diff = 0.02;
                let new_scale: f32 = rng.gen_range((1. - scale_diff)..(1. + scale_diff));
                *transform = Transform::from_scale(Vec3::splat(new_scale));
            }
        }
    }
}
