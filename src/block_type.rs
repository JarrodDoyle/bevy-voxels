use bevy::{asset::Asset, reflect::TypePath, utils::HashMap};

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct Block {
    pub identifier: String,
    pub model: Option<String>,
    pub textures: HashMap<String, String>,
}

pub type BlockType = u32;
