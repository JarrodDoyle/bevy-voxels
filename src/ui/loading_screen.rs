use bevy::prelude::*;

use crate::{screens::Screen, ui::theme::prelude::*};

pub struct LoadingScreenUiPlugin;

impl Plugin for LoadingScreenUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);
    }
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((StateScoped(Screen::Loading), Camera2d));

    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .with_children(|children| {
            children.label("Loading...").insert(Node {
                justify_content: JustifyContent::Center,
                ..default()
            });
        });
}
