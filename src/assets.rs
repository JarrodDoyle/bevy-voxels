use bevy::{
    asset::{AssetPath, Handle},
    ecs::system::Resource,
    prelude::*,
    utils::HashMap,
};
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::MapKey};

use crate::{block_type::Block, model::Model};

// TODO: Use collections as maps
#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models", collection(typed))]
    pub folder: Vec<Handle<Model>>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/blocks", collection(typed, mapped))]
    pub blocks: HashMap<AssetFileStem, Handle<Image>>,
}

#[derive(AssetCollection, Resource)]
pub struct BlockAssets {
    #[asset(path = "blocks", collection(typed, mapped))]
    pub folder: HashMap<AssetFileStem, Handle<Block>>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetFileStem(pub String);
impl MapKey for AssetFileStem {
    #[inline]
    fn from_asset_path(path: &AssetPath) -> Self {
        Self(
            path.path()
                .file_stem()
                .unwrap()
                .to_str()
                .expect("Path should be valid UTF-8")
                .into(),
        )
    }
}

#[derive(Resource)]
pub struct BlockArrayTextureHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct BlockTextureIds(pub HashMap<String, u32>);
