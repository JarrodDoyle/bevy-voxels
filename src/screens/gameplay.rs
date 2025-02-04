use bevy::prelude::*;

use crate::screens::Screen;

const GAMEPLAY_BACKGROUND_COLOR: Color = Color::srgb(0.55, 0.83, 0.9);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_gameplay_screen);
}

fn spawn_gameplay_screen(mut commands: Commands) {
    commands.insert_resource(ClearColor(GAMEPLAY_BACKGROUND_COLOR));
}
