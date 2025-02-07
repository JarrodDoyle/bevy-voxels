mod block_type;
mod model;
mod registry;

use bevy::prelude::*;

pub use {
    block_type::{Block, BlockType},
    model::Model,
    registry::Registry,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(registry::RegistryPlugin);
    }
}
