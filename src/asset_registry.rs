use bevy::{
    asset::{AssetPath, Handle, RenderAssetUsages},
    ecs::system::Resource,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::MapKey};

use crate::{block_type::Block, model::Model};

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

#[derive(AssetCollection, Resource)]
pub struct AssetRegistry {
    #[asset(path = "models", collection(typed))]
    model_handles: Vec<Handle<Model>>,
    #[asset(path = "textures/blocks", collection(typed, mapped))]
    block_texture_handles: HashMap<AssetFileStem, Handle<Image>>,
    #[asset(path = "blocks", collection(typed))]
    block_handles: Vec<Handle<Block>>,

    pub block_array_texture: Handle<Image>,
    texture_name_to_id: HashMap<String, u32>,
    block_name_to_id: HashMap<String, u32>,
    model_name_to_id: HashMap<String, u32>,
}

impl AssetRegistry {
    pub fn get_block_id(&self, name: &str) -> u32 {
        self.block_name_to_id[name]
    }

    pub fn get_block_handle_by_id(&self, id: u32) -> Handle<Block> {
        self.block_handles[id as usize].clone()
    }

    pub fn get_texture_id(&self, name: &str) -> u32 {
        self.texture_name_to_id[name]
    }

    pub fn get_model_id(&self, name: &str) -> u32 {
        self.model_name_to_id[name]
    }

    pub fn get_model_handle(&self, name: &str) -> Handle<Model> {
        self.model_handles[self.get_model_id(name) as usize].clone()
    }

    fn post_load(
        &mut self,
        asset_server: Res<AssetServer>,
        images: Res<Assets<Image>>,
        blocks: Res<Assets<Block>>,
        models: Res<Assets<Model>>,
    ) {
        self.create_block_texture_array_and_name_id_map(asset_server, images);
        self.create_block_name_to_id_map(blocks);
        self.create_model_name_to_id_map(models);
    }

    fn create_block_texture_array_and_name_id_map(
        &mut self,
        asset_server: Res<AssetServer>,
        images: Res<Assets<Image>>,
    ) {
        const SIZE: u32 = 16;
        let texture_count = self.block_texture_handles.len();
        let mut image = Image::new_fill(
            Extent3d {
                width: SIZE,
                height: SIZE * texture_count as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[255, 255, 255, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );

        self.texture_name_to_id.clear();
        for (z, k) in self.block_texture_handles.keys().enumerate() {
            self.texture_name_to_id.insert(k.0.clone(), z as u32);

            let t = images.get(self.block_texture_handles[k].id()).unwrap();
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let c = t.get_color_at(x, y).unwrap();
                    let _ = image.set_color_at(x, y + z as u32 * SIZE, c);
                }
            }
        }

        image.reinterpret_stacked_2d_as_array(texture_count as u32);
        self.block_array_texture = asset_server.add(image);
    }

    fn create_block_name_to_id_map(&mut self, blocks: Res<Assets<Block>>) {
        self.block_name_to_id.clear();
        for i in 0..self.block_handles.len() {
            let b = blocks.get(self.block_handles[i].id()).unwrap();
            self.block_name_to_id.insert(b.identifier.clone(), i as u32);
        }
    }

    fn create_model_name_to_id_map(&mut self, models: Res<Assets<Model>>) {
        self.model_name_to_id.clear();
        for i in 0..self.model_handles.len() {
            let m = models.get(self.model_handles[i].id()).unwrap();
            self.model_name_to_id.insert(m.identifier.clone(), i as u32);
        }
    }
}

pub fn construct_asset_registry(
    mut registry: ResMut<AssetRegistry>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    blocks: Res<Assets<Block>>,
    models: Res<Assets<Model>>,
) {
    registry.post_load(asset_server, images, blocks, models);
}
