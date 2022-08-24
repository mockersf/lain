pub(crate) mod builder;
pub(crate) mod heightmap;
pub(crate) mod switcher;
pub(crate) mod terra;
pub(crate) mod terrain_spawner;
pub(crate) mod ui;

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) enum PlayingState {
    Playing,
    SwitchingPlane,
    Building,
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(PlayingState::Playing)
            .add_plugin(crate::game::terrain_spawner::TerrainSpawnerPlugin)
            .add_plugin(crate::game::terra::TerraPlugin)
            .add_plugin(crate::game::switcher::Plugin)
            .add_plugin(crate::game::ui::Plugin)
            .add_plugin(crate::game::builder::Plugin);
    }
}
