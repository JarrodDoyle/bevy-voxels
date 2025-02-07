use bevy::{prelude::*, utils::HashMap};

use crate::assets::Registry;

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct Block {
    pub identifier: String,
    pub model: Option<String>,
    pub textures: HashMap<String, String>,

    #[serde(skip_deserializing)]
    pub texture_ids: HashMap<String, u32>,
}

pub type BlockType = u32;

pub fn set_block_texture_id_maps(registry: Res<Registry>, mut blocks: ResMut<Assets<Block>>) {
    for (_, block) in blocks.iter_mut() {
        for (k, v) in &block.textures {
            block
                .texture_ids
                .insert(k.to_owned(), registry.get_texture_id(v));
        }
    }
}
