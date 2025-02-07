mod block;
mod model;
mod registry;

use bevy::prelude::*;

pub use {
    block::{Block, BlockType},
    model::Model,
    registry::Registry,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(registry::RegistryPlugin);
    }
}
