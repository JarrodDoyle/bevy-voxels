use std::time::Instant;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

use crate::{
    assets::Registry,
    screens::Screen,
    world::{Chunk, VoxelWorld},
    AppSet,
};

pub struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<ArrayTextureMaterial>::default(),));
        app.add_systems(
            Update,
            (sys_chunk_mesher)
                .in_set(AppSet::Update)
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

#[derive(Component)]
pub struct ChunkNeedsMeshing;

pub const ATTRIBUTE_TEXTURE: MeshVertexAttribute =
    MeshVertexAttribute::new("texure_id", 988540917, VertexFormat::Uint32);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

const SHADER_ASSET_PATH: &str = "shaders/array_texture.wgsl";
impl Material for ArrayTextureMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TEXTURE.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

fn sys_chunk_mesher(
    mut commands: Commands,
    registry: Res<Registry>,
    storage: Res<VoxelWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
    chunks_query: Query<(Entity, &Chunk, &ChunkNeedsMeshing)>,
) {
    let material_handle = materials.add(ArrayTextureMaterial {
        array_texture: registry.block_array_texture.clone(),
    });

    const NEIGHBOUR_OFFSETS: [[i32; 3]; 6] = [
        [-1, 0, 0], // left
        [1, 0, 0],  // right
        [0, 1, 0],  // up
        [0, -1, 0], // down
        [0, 0, 1],  // front
        [0, 0, -1], // back
    ];

    let mut total_us = 0;
    let mut chunk_count = 0;
    for (id, chunk, _) in &chunks_query {
        let start_time = Instant::now();

        let cx = chunk.world_pos[0];
        let cy = chunk.world_pos[1];
        let cz = chunk.world_pos[2];

        let chunk_voxels = storage.get_chunk(&chunk.world_pos);
        let left_chunk_voxels = storage.get_chunk(&[cx - 1, cy, cz]);
        let right_chunk_voxels = storage.get_chunk(&[cx + 1, cy, cz]);
        let up_chunk_voxels = storage.get_chunk(&[cx, cy + 1, cz]);
        let down_chunk_voxels = storage.get_chunk(&[cx, cy - 1, cz]);
        let front_chunk_voxels = storage.get_chunk(&[cx, cy, cz + 1]);
        let back_chunk_voxels = storage.get_chunk(&[cx, cy, cz - 1]);

        let mut vs: Vec<[f32; 3]> = vec![];
        let mut ns = vec![];
        let mut uvs = vec![];
        let mut ts = vec![];

        let mut idx = 0;
        for z in 0..storage.chunk_len {
            for y in 0..storage.chunk_len {
                for x in 0..storage.chunk_len {
                    let block_id = chunk_voxels.unwrap()[idx];
                    let block = registry.get_block(block_id);

                    idx += 1;
                    if block.model.is_none() {
                        continue;
                    }

                    let mut cull = [false; 6];
                    for i in 0..NEIGHBOUR_OFFSETS.len() {
                        let offset = NEIGHBOUR_OFFSETS[i];

                        let mut n_local_x = x as i32 + offset[0];
                        let mut n_local_y = y as i32 + offset[1];
                        let mut n_local_z = z as i32 + offset[2];

                        let neighbor_chunk = if n_local_x < 0 {
                            n_local_x += storage.chunk_len as i32;
                            left_chunk_voxels
                        } else if n_local_x >= storage.chunk_len as i32 {
                            n_local_x -= storage.chunk_len as i32;
                            right_chunk_voxels
                        } else if n_local_y < 0 {
                            n_local_y += storage.chunk_len as i32;
                            down_chunk_voxels
                        } else if n_local_y >= storage.chunk_len as i32 {
                            n_local_y -= storage.chunk_len as i32;
                            up_chunk_voxels
                        } else if n_local_z < 0 {
                            n_local_z += storage.chunk_len as i32;
                            back_chunk_voxels
                        } else if n_local_z >= storage.chunk_len as i32 {
                            n_local_z -= storage.chunk_len as i32;
                            front_chunk_voxels
                        } else {
                            chunk_voxels
                        };

                        if neighbor_chunk.is_none() {
                            continue;
                        }

                        let n_block_id = neighbor_chunk.unwrap()[storage.local_pos_to_idx(
                            n_local_x as usize,
                            n_local_y as usize,
                            n_local_z as usize,
                        )];

                        if registry.get_block(n_block_id).model == block.model {
                            cull[i] = true;
                        }
                    }

                    let model = registry.get_model(block.model.unwrap());
                    model.mesh(
                        &cull,
                        &[x as f32, y as f32, z as f32],
                        &mut vs,
                        &mut ns,
                        &mut uvs,
                        &mut ts,
                        block,
                    );
                }
            }
        }

        let quad_count = vs.len() / 4;
        let mut is = Vec::with_capacity(6 * quad_count);
        for i in 0..quad_count as u32 {
            is.push(i * 4);
            is.push(i * 4 + 1);
            is.push(i * 4 + 2);
            is.push(i * 4);
            is.push(i * 4 + 2);
            is.push(i * 4 + 3);
        }

        if !vs.is_empty() {
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, ns)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_attribute(ATTRIBUTE_TEXTURE, ts)
            .with_inserted_indices(Indices::U32(is));
            commands.entity(id).remove::<ChunkNeedsMeshing>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material_handle.clone()),
            ));
        } else {
            // TODO: Remove meshmaterial?
            commands.entity(id).remove::<(Mesh3d, ChunkNeedsMeshing)>();
        }

        total_us += (Instant::now() - start_time).as_micros();
        chunk_count += 1;
    }

    if total_us != 0 {
        info!(
            "Meshed {chunk_count} chunks in {total_us}. Avg: {}",
            total_us / chunk_count
        );
    }
}
