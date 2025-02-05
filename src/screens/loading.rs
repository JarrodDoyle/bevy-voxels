//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_common_assets::ron::RonAssetPlugin;

use crate::{
    asset_registry::{construct_asset_registry, AssetRegistry},
    block_type::{set_block_texture_id_maps, Block},
    model::Model,
    screens::Screen,
    theme::prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        RonAssetPlugin::<Model>::new(&["model.ron"]),
        RonAssetPlugin::<Block>::new(&["block.ron"]),
    ));
    app.add_loading_state(
        LoadingState::new(Screen::Loading)
            .continue_to_state(Screen::Gameplay)
            .load_collection::<AssetRegistry>(),
    );
    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);
    app.add_systems(
        OnExit(Screen::Loading),
        (construct_asset_registry, set_block_texture_id_maps).chain(),
    );
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
