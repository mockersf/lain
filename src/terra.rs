use bevy::prelude::Plugin;
use rand::{thread_rng, Rng};

pub struct TerraPlugin;

#[derive(Clone, Copy)]
pub struct TerraNoises {
    pub elevation_seed: u32,
    pub moisture_seed: u32,
}

impl TerraNoises {
    fn new() -> Self {
        Self {
            elevation_seed: thread_rng().gen(),
            moisture_seed: thread_rng().gen(),
        }
    }
}

impl Plugin for TerraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TerraNoises::new());
    }
}
