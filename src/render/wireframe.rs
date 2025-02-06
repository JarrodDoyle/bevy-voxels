use bevy::{pbr::wireframe as bevy_wireframe, prelude::*};

pub struct WireframePlugin;

impl Plugin for WireframePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_wireframe::WireframePlugin);
        app.insert_resource(bevy_wireframe::WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        });
    }
}
