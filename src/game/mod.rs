use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;

mod chunk;
mod mesh;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((PlayerPlugin, chunk::plugin, mesh::plugin));
}
