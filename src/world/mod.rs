mod chunk;

use bevy::prelude::*;

pub use chunk::{Chunk, VoxelStorage};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(chunk::ChunkPlugin);
    }
}
