// disable console opening on windows
#![windows_subsystem = "windows"]
#![allow(clippy::needless_update, clippy::too_many_arguments)]

use std::f32::consts::FRAC_PI_4;

use bevy::{app::AppExit, prelude::*, render::texture::ImageSettings};
use bevy_jornet::JornetPlugin;
use bevy_mod_raycast::{DefaultRaycastingPlugin, RayCastSource};
use game::terrain_spawner::RaycastSet;

mod assets;
mod game;
mod lost;
mod menu;
mod splash;
mod ui_helper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = App::new();

    builder
        .insert_resource(WindowDescriptor {
            title: "Lain".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.01)));

    if cfg!(debug_assertions) {
        builder.insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::INFO,
            filter: "gfx_backend_metal=warn,wgpu_core=warn,bevy_render=info,lain=debug,bevy_render::render_resource::pipeline_cache=debug".to_string(),
        });
    } else {
        builder.insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        });
    }

    builder.add_plugins_with(DefaultPlugins, |group| {
        #[cfg(feature = "bundled")]
        group.add_before::<bevy::asset::AssetPlugin, _>(bevy_embedded_assets::EmbeddedAssetPlugin);
        group
    });

    builder
        .add_plugin(::bevy_easings::EasingsPlugin)
        .add_plugin(bevy_ninepatch::NinePatchPlugin::<()>::default());

    if cfg!(debug_assertions) {
        builder
            .add_plugin(::bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugin(::bevy::diagnostic::LogDiagnosticsPlugin::filtered(vec![
                ::bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS,
            ]));
    }

    builder.insert_resource(ImageSettings::default_nearest());

    builder
        // .insert_resource(ReportExecutionOrderAmbiguities)
        .add_plugin(JornetPlugin::with_leaderboard(
            "8e6c264a-a372-4e65-a994-e236db4dba55",
            "daf527c7-eca7-42b8-86b9-dddd1d93eaf1",
        ))
        // game management
        .add_startup_system(general_setup)
        // ui
        .add_plugin(crate::ui_helper::button::Plugin)
        .add_plugin(DefaultRaycastingPlugin::<RaycastSet>::default())
        // screens
        .add_state(GameState::Splash)
        .add_state_to_stage(CoreStage::PostUpdate, GameState::Splash)
        .add_system_set(SystemSet::on_enter(GameState::Exit).with_system(exit))
        .add_plugin(crate::assets::AssetPlugin)
        .add_plugin(crate::splash::Plugin)
        .add_plugin(crate::menu::Plugin)
        .add_plugin(crate::game::Plugin)
        .add_plugin(crate::lost::Plugin)
        .add_system(animate_light_direction);

    if let Some(leaderboard_id) = option_env!("JORNET_LEADERBOARD_ID") {
        builder.add_plugin(JornetPlugin::with_leaderboard(
            leaderboard_id,
            option_env!("JORNET_LEADERBOARD_KEY").unwrap_or_default(),
        ));
    }
    #[cfg(feature = "debug-graph")]
    bevy_mod_debugdump::print_schedule(&mut builder);

    #[cfg(not(feature = "debug-graph"))]
    builder.run();

    Ok(())
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) enum GameState {
    Splash,
    Menu,
    // About,
    Playing,
    // Paused,
    Lost,
    Exit,
}

fn general_setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 10.0)),
            ..default()
        })
        .insert(RayCastSource::<RaycastSet>::new());
}

fn exit(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.seconds_since_startup() as f32 * std::f32::consts::TAU / 10000.0,
            -FRAC_PI_4,
        );
    }
}
