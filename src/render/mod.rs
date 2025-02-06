mod chunk_mesh;
mod sky;
mod wireframe;

use bevy::prelude::*;

pub use chunk_mesh::ChunkNeedsMeshing;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sky::SkyPlugin,
            chunk_mesh::ChunkMeshPlugin,
            wireframe::WireframePlugin,
        ));
    }
}
