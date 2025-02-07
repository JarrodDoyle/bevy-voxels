use bevy::{prelude::*, utils::HashMap};

pub type BlockType = usize;

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct BlockDef {
    pub identifier: String,
    pub model: Option<String>,
    pub textures: HashMap<String, String>,
}

pub struct Block {
    pub identifier: String,
    pub model: Option<usize>,
    pub textures: HashMap<String, usize>,
}
