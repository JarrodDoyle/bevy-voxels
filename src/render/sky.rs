use bevy::prelude::*;

use crate::screens::Screen;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), set_clear_colour);
    }
}

fn set_clear_colour(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::srgb(0.55, 0.83, 0.9)));
}
