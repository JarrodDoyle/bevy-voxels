use bevy::{asset::Handle, ecs::system::Resource, prelude::*};
use bevy_asset_loader::asset_collection::AssetCollection;

use crate::model::Model;

// TODO: Use collections as maps
#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models", collection(typed))]
    pub folder: Vec<Handle<Model>>,
}
