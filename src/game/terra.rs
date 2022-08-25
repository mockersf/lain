use bevy::prelude::Plugin;
use rand::{thread_rng, Rng};

pub(crate) struct TerraPlugin;

#[derive(Clone, Copy)]
pub(crate) struct TerraNoises {
    pub(crate) material_seed: u32,
}

impl TerraNoises {
    fn new() -> Self {
        Self {
            material_seed: thread_rng().gen(),
        }
    }
}

impl Plugin for TerraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TerraNoises::new());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Plane {
    Material,
    Ethereal,
}

impl Plane {
    pub(crate) fn next(&self) -> Self {
        match self {
            Plane::Material => Plane::Ethereal,
            Plane::Ethereal => Plane::Material,
        }
    }
}
