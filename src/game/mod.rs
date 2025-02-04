use bevy::prelude::*;

mod chunk;
mod mesh;
mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, chunk::plugin, mesh::plugin));
}
