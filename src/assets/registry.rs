use bevy::{
    asset::{AssetPath, Handle, RenderAssetUsages},
    ecs::system::Resource,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
    mapped::MapKey,
};
use bevy_common_assets::ron::RonAssetPlugin;

use crate::screens::Screen;

use super::{
    block::{Block, BlockDef},
    model::ModelDef,
    Model,
};

pub struct RegistryPlugin;

impl Plugin for RegistryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<ModelDef>::new(&["model.ron"]),
            RonAssetPlugin::<BlockDef>::new(&["block.ron"]),
        ));
        app.add_loading_state(
            LoadingState::new(Screen::Loading)
                .continue_to_state(Screen::Gameplay)
                .load_collection::<Registry>(),
        );
        app.add_systems(OnExit(Screen::Loading), (construct_asset_registry).chain());
    }
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

#[derive(AssetCollection, Resource)]
pub struct Registry {
    #[asset(path = "textures/blocks", collection(typed, mapped))]
    block_texture_handles: HashMap<AssetFileStem, Handle<Image>>,
    texture_name_to_id: HashMap<String, usize>,
    pub block_array_texture: Handle<Image>,

    #[asset(path = "models", collection(typed))]
    model_def_handles: Vec<Handle<ModelDef>>,
    model_name_to_id: HashMap<String, usize>,
    models: Vec<Model>,

    #[asset(path = "blocks", collection(typed))]
    block_def_handles: Vec<Handle<BlockDef>>,
    block_name_to_id: HashMap<String, usize>,
    blocks: Vec<Block>,
}

// TODO: Make these return Options
impl Registry {
    pub fn get_texture_id(&self, name: &str) -> usize {
        self.texture_name_to_id[name]
    }

    pub fn get_block(&self, id: usize) -> &Block {
        &self.blocks[id]
    }

    pub fn get_block_id(&self, name: &str) -> usize {
        self.block_name_to_id[name]
    }

    pub fn get_model(&self, id: usize) -> &Model {
        &self.models[id]
    }

    pub fn get_model_id(&self, name: &str) -> usize {
        self.model_name_to_id[name]
    }

    fn post_load(
        &mut self,
        asset_server: Res<AssetServer>,
        images: Res<Assets<Image>>,
        block_defs: Res<Assets<BlockDef>>,
        model_defs: Res<Assets<ModelDef>>,
    ) {
        // Set up id maps
        self.texture_name_to_id.clear();
        for (z, k) in self.block_texture_handles.keys().enumerate() {
            self.texture_name_to_id.insert(k.0.clone(), z);
        }

        self.block_name_to_id.clear();
        for i in 0..self.block_def_handles.len() {
            let b = block_defs.get(self.block_def_handles[i].id()).unwrap();
            self.block_name_to_id.insert(b.identifier.clone(), i);
        }

        self.model_name_to_id.clear();
        for i in 0..self.model_def_handles.len() {
            let m = model_defs.get(self.model_def_handles[i].id()).unwrap();
            self.model_name_to_id.insert(m.identifier.clone(), i);
        }

        // Create block array texture
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

        for (z, k) in self.block_texture_handles.keys().enumerate() {
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

        // Create models and blocks
        for i in 0..self.model_def_handles.len() {
            let model_def = model_defs.get(self.model_def_handles[i].id()).unwrap();
            let model = Model {
                identifier: model_def.identifier.clone(),
                faces: model_def.faces.clone(),
            };
            self.models.push(model);
        }

        for i in 0..self.block_def_handles.len() {
            let block_def = block_defs.get(self.block_def_handles[i].id()).unwrap();
            let model = block_def.model.as_ref().map(|name| self.get_model_id(name));
            let mut textures = HashMap::new();
            for (k, v) in &block_def.textures {
                textures.insert(k.to_owned(), self.get_texture_id(v));
            }
            let block = Block {
                identifier: block_def.identifier.clone(),
                model,
                textures,
            };
            self.blocks.push(block);
        }
    }
}

fn construct_asset_registry(
    mut registry: ResMut<Registry>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    blocks: Res<Assets<BlockDef>>,
    models: Res<Assets<ModelDef>>,
) {
    registry.post_load(asset_server, images, blocks, models);
}
