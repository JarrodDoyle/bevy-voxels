use bevy::{prelude::*, utils::HashMap};

#[derive(serde::Deserialize, Asset, TypePath)]
pub struct Model {
    identifier: String,
    faces: Vec<Face>,
}

impl Model {
    // TODO: correct textures
    pub fn mesh(
        &self,
        cull: &[bool; 6],
        offset: &[f32; 3],
        vs: &mut Vec<[f32; 3]>,
        ns: &mut Vec<[f32; 3]>,
        uvs: &mut Vec<[f32; 2]>,
        ts: &mut Vec<u32>,
        is: &mut Vec<u32>,
        texture_map: &HashMap<String, u32>,
    ) {
        let default_t = *texture_map.get("default").unwrap();

        let f_len = self.faces.len();
        for i in 0..f_len {
            let face = &self.faces[i];
            if face.cull.is_none_or(|c| cull[c]) {
                continue;
            }

            let t = if texture_map.contains_key(&face.texture) {
                texture_map[&face.texture]
            } else {
                default_t
            };

            let vcount = vs.len() as u32;
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
                ts.push(t);
            }

            for j in 1..(fv_len as u32 - 1) {
                is.push(vcount);
                is.push(vcount + j);
                is.push(vcount + j + 1);
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct Face {
    texture: String,
    normal: [f32; 3],
    vertices: Vec<Vertex>,
    cull: Option<usize>,
}

#[derive(serde::Deserialize)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}
