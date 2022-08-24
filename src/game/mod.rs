pub(crate) mod heightmap;
pub(crate) mod switcher;
pub(crate) mod terra;
pub(crate) mod terrain_spawner;
pub(crate) mod ui;
    Playing,
    SwitchingPlane,
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_state(PlayingState::Playing)
            .add_plugin(crate::game::terrain_spawner::TerrainSpawnerPlugin)
            .add_plugin(crate::game::terra::TerraPlugin)
