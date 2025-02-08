use std::{
    fs::File,
    io::{BufReader, Read, Write},
};

use bevy::prelude::*;

use crate::{assets::Registry, render::ChunkNeedsMeshing, screens::Screen, AppSet};

use super::voxel_world::VoxelWorld;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::Gameplay),
            (sys_chunk_spawner).after(AppSet::TickTimers),
        );
        app.add_systems(
            Update,
            (
                sys_mark_save_all,
                sys_save_chunks,
                sys_mark_load_all,
                sys_load_chunks,
            )
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

#[derive(Component)]
pub struct Chunk {
    pub world_pos: [i32; 3],
}

#[derive(Component)]
pub struct ChunkNeedsSaving;

#[derive(Component)]
pub struct ChunkNeedsLoading;

fn sys_chunk_spawner(
    mut commands: Commands,
    mut storage: ResMut<VoxelWorld>,
    registry: Res<Registry>,
) {
    let voxels_per_chunk = storage.chunk_len * storage.chunk_len * storage.chunk_len;
    let mut noise_vals = vec![0.0; voxels_per_chunk];

    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
                storage.terrain_noise.gen_uniform_grid_3d(
                    &mut noise_vals,
                    (storage.chunk_len * x) as i32,
                    (storage.chunk_len * y) as i32,
                    (storage.chunk_len * z) as i32,
                    storage.chunk_len as i32,
                    storage.chunk_len as i32,
                    storage.chunk_len as i32,
                    storage.terrain_frequency,
                    storage.terrain_seed,
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

fn sys_mark_save_all(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    query_chunks: Query<Entity, (With<Chunk>, Without<ChunkNeedsSaving>)>,
) {
    if keys.just_pressed(KeyCode::KeyO) {
        for id in &query_chunks {
            commands.entity(id).insert(ChunkNeedsSaving);
        }
    }
}

fn sys_mark_load_all(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    query_chunks: Query<Entity, (With<Chunk>, Without<ChunkNeedsSaving>)>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        for id in &query_chunks {
            commands.entity(id).insert(ChunkNeedsLoading);
        }
    }
}

fn sys_save_chunks(
    mut commands: Commands,
    voxel_world: Res<VoxelWorld>,
    query_chunks: Query<(Entity, &Chunk), With<ChunkNeedsSaving>>,
) {
    for (id, chunk) in &query_chunks {
        let data = voxel_world.get_chunk(&chunk.world_pos).unwrap();
        let buffer = bincode::serialize(data).unwrap();

        let path = format!(
            "./saves/w1/{}_{}_{}.dat",
            chunk.world_pos[0], chunk.world_pos[1], chunk.world_pos[2]
        );
        let mut f = File::create(&path).unwrap();

        f.write_all(&buffer).unwrap();

        commands.entity(id).remove::<ChunkNeedsSaving>();
    }
}

fn sys_load_chunks(
    mut commands: Commands,
    mut voxel_world: ResMut<VoxelWorld>,
    query_chunks: Query<(Entity, &Chunk), With<ChunkNeedsLoading>>,
) {
    for (id, chunk) in &query_chunks {
        let path = format!(
            "./saves/w1/{}_{}_{}.dat",
            chunk.world_pos[0], chunk.world_pos[1], chunk.world_pos[2]
        );

        let f = File::open(&path).unwrap();
        let mut reader = BufReader::new(f);
        let mut binary_buffer = vec![];
        reader.read_to_end(&mut binary_buffer).unwrap();

        let buffer = bincode::deserialize(&binary_buffer).unwrap();
        let data = voxel_world.get_chunk_mut(&chunk.world_pos).unwrap();
        *data = buffer;

        commands.entity(id).remove::<ChunkNeedsLoading>();
        commands.entity(id).insert(ChunkNeedsMeshing);
    }
}
