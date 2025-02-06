use bevy::prelude::*;

use crate::screens::Screen;

pub struct CrosshairUiPlugin;

impl Plugin for CrosshairUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), spawn_crosshair);
    }
}

fn spawn_crosshair(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crosshair = asset_server.load("images/crosshair.png");
    commands
        .spawn((
            StateScoped(Screen::Gameplay),
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode {
                    image: crosshair,
                    ..default()
                },
                Node {
                    width: Val::Px(32.0),
                    ..default()
                },
            ));
        });
}
