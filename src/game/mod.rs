use bevy::prelude::*;

mod chunk;
pub mod player;

pub use chunk::{Chunk, VoxelStorage};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, chunk::plugin));
}
