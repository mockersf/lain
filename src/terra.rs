use bevy::prelude::Plugin;
use rand::{thread_rng, Rng};

pub struct TerraPlugin;

#[derive(Clone, Copy)]
pub struct TerraNoises {
    pub material_seed: u32,
    pub ethereal_seed: u32,
}

impl TerraNoises {
    fn new() -> Self {
        Self {
            material_seed: thread_rng().gen(),
            ethereal_seed: thread_rng().gen(),
        }
    }
}

impl Plugin for TerraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TerraNoises::new());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    Material,
    Ethereal,
}
