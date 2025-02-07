use bevy::prelude::*;

use super::block::Block;

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct ModelDef {
    pub identifier: String,
    pub faces: Vec<Face>,
}

pub struct Model {
    pub identifier: String,
    pub faces: Vec<Face>,
}

impl Model {
    pub fn mesh(
        &self,
        cull: &[bool; 6],
        offset: &[f32; 3],
        vs: &mut Vec<[f32; 3]>,
        ns: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        ts: &mut Vec<u32>,
        block: &Block,
    ) {
        let mut first_unculled = -1;
        for i in 0..self.faces.len() {
            if self.faces[i].cull.is_none_or(|c| !cull[c]) {
                first_unculled = i as i32;
                break;
            }
        }

        if first_unculled == -1 {
            return;
        }

        let default_t = block.textures["default"];

        let f_len = self.faces.len();
        for i in 0..f_len {
            let face = &self.faces[i];
            if face.cull.is_some_and(|c| cull[c]) {
                continue;
            }

            let t = if block.textures.contains_key(&face.texture) {
                block.textures[&face.texture]
            } else {
                default_t
            };

            let fv_len = face.vertices.len();
            for j in 0..fv_len {
                let v = &face.vertices[j];
                vs.push([
                    offset[0] + v.position[0],
                    offset[1] + v.position[1],
                    offset[2] + v.position[2],
                ]);
                ns.push(face.normal);
                uvs.push(v.uv);
                ts.push(t as u32);
            }
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Face {
    pub texture: String,
    pub normal: [f32; 3],
    pub vertices: Vec<Vertex>,
    pub cull: Option<usize>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}
