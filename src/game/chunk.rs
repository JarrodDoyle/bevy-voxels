use std::time::Instant;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    utils::HashMap,
};
use fastnoise2::SafeNode;

use crate::{
    asset_registry::AssetRegistry,
    block_type::{Block, BlockType},
    game::mesh::ATTRIBUTE_TEXTURE,
    model::Model,
    screens::Screen,
    AppSet,
};

use super::mesh::ArrayTextureMaterial;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (setup_noise, sys_chunk_spawner).chain(),
    );
    app.add_systems(
        Update,
        (sys_chunk_mesher)
            .in_set(AppSet::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource)]
struct WorldNoise {
    terrain: SafeNode,
}

#[derive(Component)]
pub struct VoxelStorage {
    pub chunk_len: usize,
    pub voxels: HashMap<[i32; 3], Vec<BlockType>>,
}

impl VoxelStorage {
    pub fn get_voxel(
        &self,
        chunk_pos: &[i32; 3],
        local_x: usize,
        local_y: usize,
        local_z: usize,
    ) -> Option<BlockType> {
        self.voxels
            .get(chunk_pos)
            .map(|chunk| chunk[self.local_pos_to_idx(local_x, local_y, local_z)])
    }

    pub fn set_voxel(
        &mut self,
        chunk_pos: &[i32; 3],
        local_x: usize,
        local_y: usize,
        local_z: usize,
        block_type: BlockType,
    ) {
        let idx = self.local_pos_to_idx(local_x, local_y, local_z);
        if let Some(chunk) = self.voxels.get_mut(chunk_pos) {
            chunk[idx] = block_type;
        }
    }

    pub fn get_chunk(&self, chunk_pos: &[i32; 3]) -> Option<&Vec<BlockType>> {
        self.voxels.get(chunk_pos)
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: &[i32; 3]) -> Option<&mut Vec<BlockType>> {
        self.voxels.get_mut(chunk_pos)
    }

    pub fn load_chunk(&mut self, chunk_pos: &[i32; 3], chunk_voxels: Vec<BlockType>) {
        assert_eq!(
            chunk_voxels.len(),
            self.chunk_len * self.chunk_len * self.chunk_len
        );

        self.voxels.insert(*chunk_pos, chunk_voxels);
    }

    pub fn local_pos_to_idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.chunk_len + z * self.chunk_len * self.chunk_len
    }
}

#[derive(Component)]
pub struct Chunk {
    pub world_pos: [i32; 3],
}

#[derive(Component)]
pub struct ChunkNeedsMeshing;

fn setup_noise(mut commands: Commands) {
    let encoded_node_tree = "DQADAAAAAAAAQCkAAAAAAD8AAAAAAA==";
    let node = SafeNode::from_encoded_node_tree(encoded_node_tree).unwrap();

    commands.insert_resource(WorldNoise { terrain: node });
}

fn sys_chunk_spawner(
    mut commands: Commands,
    world_noise: Res<WorldNoise>,
    registry: Res<AssetRegistry>,
) {
    let mut storage = VoxelStorage {
        chunk_len: 32,
        voxels: HashMap::<[i32; 3], Vec<BlockType>>::new(),
    };

    const FREQUENCY: f32 = 0.005;
    const SEED: i32 = 1338;

    let voxels_per_chunk = storage.chunk_len * storage.chunk_len * storage.chunk_len;
    let mut noise_vals = vec![0.0; voxels_per_chunk];

    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                world_noise.terrain.gen_uniform_grid_3d(
                    &mut noise_vals,
                    (storage.chunk_len * x) as i32,
                    (storage.chunk_len * y) as i32,
                    (storage.chunk_len * z) as i32,
                    storage.chunk_len as i32,
                    storage.chunk_len as i32,
                    storage.chunk_len as i32,
                    FREQUENCY,
                    SEED,
                );

                let mut chunk_voxels = vec![registry.get_block_id("air"); voxels_per_chunk];
                (0..voxels_per_chunk).for_each(|i| {
                    if noise_vals[i] > 0. {
                        chunk_voxels[i] = registry.get_block_id("stone");
                    }
                });

                for z in 0..storage.chunk_len {
                    for y in 0..storage.chunk_len {
                        for x in 0..storage.chunk_len {
                            let i = storage.local_pos_to_idx(x, y, z);
                            if chunk_voxels[i] == registry.get_block_id("air") {
                                if y > 0
                                    && chunk_voxels[storage.local_pos_to_idx(x, y - 1, z)]
                                        != registry.get_block_id("air")
                                    && chunk_voxels[storage.local_pos_to_idx(x, y - 1, z)]
                                        != registry.get_block_id("stone_fence")
                                {
                                    chunk_voxels[i] = registry.get_block_id("stone_fence");
                                }

                                continue;
                            }

                            for dy in 1..4 {
                                if y + dy < storage.chunk_len
                                    && chunk_voxels[storage.local_pos_to_idx(x, y + dy, z)]
                                        == registry.get_block_id("air")
                                {
                                    chunk_voxels[i] = if dy == 1 {
                                        registry.get_block_id("grass")
                                    } else {
                                        registry.get_block_id("dirt")
                                    };
                                    break;
                                }
                            }
                        }
                    }
                }

                let world_pos = [x as i32, y as i32, z as i32];
                storage.load_chunk(&world_pos, chunk_voxels);

                commands.spawn((
                    StateScoped(Screen::Gameplay),
                    Chunk { world_pos },
                    ChunkNeedsMeshing,
                    Transform::from_xyz(x as f32 * 32., y as f32 * 32., z as f32 * 32.),
                ));
            }
        }
    }

    commands.spawn((StateScoped(Screen::Gameplay), storage));
}

fn sys_chunk_mesher(
    mut commands: Commands,
    registry: Res<AssetRegistry>,
    models: Res<Assets<Model>>,
    blocks: Res<Assets<Block>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
    query_storage: Query<&VoxelStorage>,
    chunks_query: Query<(Entity, &Chunk, &ChunkNeedsMeshing)>,
) {
    let voxel_storage = query_storage.single();

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

    let get_neighbour_pos =
        |chunk_len: usize, chunk_pos: &[i32; 3], local_pos: &[i32; 3], offset: &[i32; 3]| {
            let mut local_x = local_pos[0] + offset[0];
            let mut local_y = local_pos[1] + offset[1];
            let mut local_z = local_pos[2] + offset[2];

            let mut new_chunk_pos = [chunk_pos[0], chunk_pos[1], chunk_pos[2]];
            if local_x < 0 {
                new_chunk_pos[0] -= 1;
                local_x += chunk_len as i32;
            }
            if local_y < 0 {
                new_chunk_pos[1] -= 1;
                local_y += chunk_len as i32;
            }
            if local_z < 0 {
                new_chunk_pos[2] -= 1;
                local_z += chunk_len as i32;
            }
            if local_x >= chunk_len as i32 {
                new_chunk_pos[0] += 1;
                local_x -= chunk_len as i32;
            }
            if local_y >= chunk_len as i32 {
                new_chunk_pos[1] += 1;
                local_y -= chunk_len as i32;
            }
            if local_z >= chunk_len as i32 {
                new_chunk_pos[2] += 1;
                local_z -= chunk_len as i32;
            }

            (
                new_chunk_pos,
                local_x as usize,
                local_y as usize,
                local_z as usize,
            )
        };

    let mut total_us = 0;
    let mut chunk_count = 0;
    for (id, chunk, _) in &chunks_query {
        let start_time = Instant::now();

        let world_pos: &[i32; 3] = &chunk.world_pos;

        let mut vs: Vec<[f32; 3]> = vec![];
        let mut ns = vec![];
        let mut uvs = vec![];
        let mut ts = vec![];
        for z in 0..voxel_storage.chunk_len {
            for y in 0..voxel_storage.chunk_len {
                for x in 0..voxel_storage.chunk_len {
                    let block = match voxel_storage.get_voxel(world_pos, x, y, z) {
                        Some(i) => blocks.get(registry.get_block_handle_by_id(i).id()),
                        None => None,
                    };

                    if block.is_none_or(|b| b.model.is_none()) {
                        continue;
                    }

                    let block = block.unwrap();

                    let mut cull = [false; 6];
                    for i in 0..NEIGHBOUR_OFFSETS.len() {
                        let offset = NEIGHBOUR_OFFSETS[i];
                        let (n_chunk_pos, n_local_x, n_local_y, n_local_z) = get_neighbour_pos(
                            voxel_storage.chunk_len,
                            world_pos,
                            &[x as i32, y as i32, z as i32],
                            &offset,
                        );

                        let n_block = match voxel_storage.get_voxel(
                            &n_chunk_pos,
                            n_local_x,
                            n_local_y,
                            n_local_z,
                        ) {
                            Some(i) => blocks.get(registry.get_block_handle_by_id(i).id()),
                            None => None,
                        };

                        if n_block.is_some_and(|b| b.model == block.model) {
                            cull[i] = true;
                        }
                    }

                    let model_name = &block.model.clone().unwrap();
                    let model = models
                        .get(registry.get_model_handle(model_name).id())
                        .unwrap();

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
