use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;

mod chunk;
mod mesh;
mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((PlayerPlugin, player::plugin, chunk::plugin, mesh::plugin));
}
