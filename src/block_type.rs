use bevy::{asset::Asset, reflect::TypePath, utils::HashMap};

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct Block {
    identifier: String,
    model: String,
    textures: HashMap<String, String>,
}
