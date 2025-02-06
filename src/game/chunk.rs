use bevy::{prelude::*, utils::HashMap};
use fastnoise2::SafeNode;

use crate::{
    asset_registry::AssetRegistry, block_type::BlockType, render::ChunkNeedsMeshing,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (setup_noise, sys_chunk_spawner).chain(),
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
