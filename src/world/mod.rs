mod chunk;
mod voxel_world;

use bevy::prelude::*;

pub use {chunk::Chunk, voxel_world::VoxelWorld};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((chunk::ChunkPlugin, voxel_world::VoxelWorldPlugin));
    }
}
