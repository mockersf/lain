// disable console opening on windows
#![windows_subsystem = "windows"]
#![allow(clippy::needless_update, clippy::too_many_arguments)]

use bevy::{app::AppExit, prelude::*, render::texture::ImageSettings};

mod assets;
mod menu;
mod splash;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = App::new();

    builder
        .insert_resource(WindowDescriptor {
            title: "Lain".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.01)));

    #[cfg(not(target_arch = "wasm32"))]
    if cfg!(debug_assertions) {
        builder.insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::INFO,
            filter: "gfx_backend_metal=warn,wgpu_core=warn,bevy_render=info".to_string(),
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
        // game management
        .add_startup_system(general_setup)
        .insert_resource(GameScreen::default())
        // ui
        .add_plugin(crate::ui::button::Plugin)
        // screens
        .add_state(GameState::Splash)
        .add_state_to_stage(CoreStage::PostUpdate, GameState::Splash)
        .add_system_set(SystemSet::on_enter(GameState::Exit).with_system(exit))
        .add_plugin(crate::assets::AssetPlugin)
        .add_plugin(crate::splash::Plugin)
        .add_plugin(crate::menu::Plugin)
        // .add_plugin(crate::about::Plugin)
        // .add_plugin(crate::game::Plugin)
        // .add_plugin(crate::lost::Plugin)
        .run();

    Ok(())
}

pub const STAGE: &str = "game";
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum GameState {
    Splash,
    Menu,
    About,
    Playing,
    Paused,
    Lost,
    Exit,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Screen {
    Splash,
    Menu,
    About,
    Game,
    Exit,
    Lost,
}

#[derive(Debug, Default)]
pub struct GameScreen {
    pub highscore: u32,
    pub highround: u16,
}

impl GameScreen {
    pub fn is_new_highscore(&self, score: u32) -> bool {
        self.highscore != 0 && score > self.highscore
    }
    pub fn is_new_highround(&self, round: u16) -> bool {
        self.highround != 0 && round > self.highround
    }
}

fn general_setup(mut commands: Commands) {
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 10.0)),
        ..default()
    });
}

fn exit(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}
