pub(crate) mod builder;
pub(crate) mod heightmap;
pub(crate) mod nests;
pub(crate) mod stats;
pub(crate) mod switcher;
pub(crate) mod terra;
pub(crate) mod terrain_spawner;
pub(crate) mod towers;
pub(crate) mod ui;
pub(crate) mod zombies;

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
            .add_plugin(stats::Plugin)
            .add_plugin(terrain_spawner::TerrainSpawnerPlugin)
            .add_plugin(terra::TerraPlugin)
            .add_plugin(switcher::Plugin)
            .add_plugin(ui::Plugin)
            .add_plugin(builder::Plugin)
            .add_plugin(nests::Plugin)
            .add_plugin(zombies::Plugin)
            .add_plugin(towers::Plugin);
    }
}
