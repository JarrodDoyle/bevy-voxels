use bevy::prelude::*;

use crate::{assets::Registry, render::ChunkNeedsMeshing, screens::Screen, AppSet};

use super::voxel_world::{VoxelStorage, WorldNoise};

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::Gameplay),
            sys_chunk_spawner.after(AppSet::TickTimers),
        );
    }
}

#[derive(Component)]
pub struct Chunk {
    pub world_pos: [i32; 3],
}

fn sys_chunk_spawner(
    mut commands: Commands,
    mut storage: ResMut<VoxelStorage>,
    world_noise: Res<WorldNoise>,
    registry: Res<Registry>,
) {
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
}
