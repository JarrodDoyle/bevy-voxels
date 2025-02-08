use bevy::prelude::*;

use crate::{
    assets::Registry,
    game::player::{Hotbar, Player},
    screens::Screen,
};

pub struct HotbarUiPlugin;

impl Plugin for HotbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Gameplay), setup);
        app.add_systems(Update, update_ui.run_if(in_state(Screen::Gameplay)));
    }
}

#[derive(Component)]
pub struct HotbarUi;

fn setup(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Gameplay),
        HotbarUi,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(16.0)),
            position_type: PositionType::Absolute,
            ..default()
        },
    ));
}

// TODO: Put models of the item
fn update_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<Registry>,
    query_hotbar_ui: Query<Entity, With<HotbarUi>>,
    query_player: Query<&Hotbar, (With<Player>, Changed<Hotbar>)>,
) {
    let hotbar_ui_id = query_hotbar_ui.single();
    let Ok(hotbar) = query_player.get_single() else {
        return;
    };

    let slot_image = asset_server.load("images/hotbar-slot.png");
    commands
        .entity(hotbar_ui_id)
        .despawn_descendants()
        .with_children(|parent| {
            if let Some(block_id) = hotbar.slots[hotbar.active_slot] {
                let block = registry.get_block(block_id);
                parent.spawn(Text::new(&block.identifier));
            }

            parent
                .spawn(Node {
                    align_items: AlignItems::End,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|parent| {
                    for _ in 0..hotbar.slots.len() {
                        parent.spawn(ImageNode {
                            image: slot_image.clone(),
                            ..default()
                        });
                    }
                });
        });
}
