use std::{
    fs::{self, File},
    io::{Read, Write},
    time::Instant,
};

use bevy::{diagnostic::Diagnostics, prelude::*};
use flate2::{read::DeflateDecoder, write::DeflateEncoder, Compression};

use crate::{
    assets::Registry,
    diagnostics::{GEN_TIME_DIAGNOSTIC, LOAD_TIME_DIAGNOSTIC, SAVE_TIME_DIAGNOSTIC},
    game::player::Player,
    render::ChunkNeedsMeshing,
    screens::Screen,
};

use super::voxel_world::VoxelWorld;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                load_unload_chunks_around_player,
                (
                    generate_chunks,
                    sys_mark_save_all,
                    (sys_save_chunks, unload_chunks).chain(),
                    sys_mark_load_all,
                    sys_load_chunks,
                    scheduled_save,
                ),
            )
                .chain()
                .run_if(in_state(Screen::Gameplay)),
        );
    }
}

#[derive(Component)]
pub struct Chunk {
    pub world_pos: [i32; 3],
    pub dirty: bool,
}

#[derive(Component)]
pub struct ChunkNeedsSaving;

#[derive(Component)]
pub struct ChunkNeedsLoading;

#[derive(Component)]
pub struct ChunkNeedsUnloading;

#[derive(Component)]
pub struct ChunkNeedsGenerating;

fn load_unload_chunks_around_player(
    mut commands: Commands,
    storage: Res<VoxelWorld>,
    query_player: Query<&Transform, With<Player>>,
    query_chunks: Query<(Entity, &Chunk)>,
) {
    let player_translate = query_player.single().translation;
    let player_chunk = [
        (player_translate.x / storage.chunk_len as f32).floor() as i32,
        (player_translate.y / storage.chunk_len as f32).floor() as i32,
        (player_translate.z / storage.chunk_len as f32).floor() as i32,
    ];

    let chunk_radius = 4;
    let load_region_side_length = chunk_radius * 2 + 1;
    let num_chunks = usize::pow(load_region_side_length, 3);
    let mut needs_spawning = vec![true; num_chunks];
    for (id, chunk) in &query_chunks {
        let mut exists = true;
        for i in 0..3 {
            if chunk.world_pos[i] < player_chunk[i] - chunk_radius as i32
                || chunk.world_pos[i] > player_chunk[i] + chunk_radius as i32
            {
                exists = false;
                break;
            }
        }

        if exists {
            let x = (chunk.world_pos[0] - (player_chunk[0] - chunk_radius as i32)) as usize;
            let y = (chunk.world_pos[1] - (player_chunk[1] - chunk_radius as i32)) as usize;
            let z = (chunk.world_pos[2] - (player_chunk[2] - chunk_radius as i32)) as usize;
            let idx = x
                + y * load_region_side_length
                + z * load_region_side_length * load_region_side_length;
            needs_spawning[idx] = false;
        } else {
            if chunk.dirty {
                commands.entity(id).insert(ChunkNeedsSaving);
            }
            commands.entity(id).insert(ChunkNeedsUnloading);
        }
    }

    let mut idx = 0;
    for z in 0..load_region_side_length {
        for y in 0..load_region_side_length {
            for x in 0..load_region_side_length {
                idx += 1;
                if !needs_spawning[idx - 1] {
                    continue;
                }

                let cx = player_chunk[0] - chunk_radius as i32 + x as i32;
                let cy = player_chunk[1] - chunk_radius as i32 + y as i32;
                let cz = player_chunk[2] - chunk_radius as i32 + z as i32;

                commands.spawn((
                    StateScoped(Screen::Gameplay),
                    Chunk {
                        world_pos: [cx, cy, cz],
                        dirty: false,
                    },
                    ChunkNeedsLoading,
                    Transform::from_xyz(cx as f32 * 32., cy as f32 * 32., cz as f32 * 32.),
                ));
            }
        }
    }
}

fn generate_chunks(
    mut commands: Commands,
    mut diagnostics: Diagnostics,
    mut storage: ResMut<VoxelWorld>,
    registry: Res<Registry>,
    mut query_chunks: Query<(Entity, &mut Chunk), With<ChunkNeedsGenerating>>,
) {
    let voxels_per_chunk = storage.chunk_len * storage.chunk_len * storage.chunk_len;
    let mut noise_vals = vec![0.0; voxels_per_chunk];

    let mut chunk_count = 0;
    for (id, mut chunk) in &mut query_chunks {
        // TODO: Make chunks per frame configurable
        if chunk_count > 8 {
            return;
        }

        let start_time = Instant::now();

        let x = chunk.world_pos[0];
        let y = chunk.world_pos[1];
        let z = chunk.world_pos[2];

        storage.terrain_noise.gen_uniform_grid_3d(
            &mut noise_vals,
            storage.chunk_len as i32 * x,
            storage.chunk_len as i32 * y,
            storage.chunk_len as i32 * z,
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

        let world_pos = [x, y, z];
        storage.load_chunk(&world_pos, chunk_voxels);

        chunk.dirty = true;
        commands
            .entity(id)
            .remove::<ChunkNeedsGenerating>()
            .insert(ChunkNeedsMeshing);

        let gen_time = (Instant::now() - start_time).as_micros();
        diagnostics.add_measurement(&GEN_TIME_DIAGNOSTIC, || gen_time as f64);
        chunk_count += 1;
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
    mut diagnostics: Diagnostics,
    voxel_world: Res<VoxelWorld>,
    mut query_chunks: Query<(Entity, &mut Chunk), With<ChunkNeedsSaving>>,
) {
    let mut chunk_count = 0;
    for (id, mut chunk) in &mut query_chunks {
        // TODO: Make chunks per frame configurable
        if chunk_count > 8 {
            return;
        }

        let start_time = Instant::now();

        let save_dir = format!("./saves/{}", voxel_world.world_name);
        let _ = fs::create_dir(&save_dir);
        let Ok(true) = fs::exists(&save_dir) else {
            continue;
        };

        let data = voxel_world.get_chunk(&chunk.world_pos).unwrap();
        let buffer = bitcode::encode(data);

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(&buffer).unwrap();
        let compressed_buffer = encoder.finish().unwrap();

        let path = format!(
            "{}/{}_{}_{}.dat",
            &save_dir, chunk.world_pos[0], chunk.world_pos[1], chunk.world_pos[2]
        );
        let mut f = File::create(&path).unwrap();
        f.write_all(&compressed_buffer).unwrap();

        chunk.dirty = false;
        commands.entity(id).remove::<ChunkNeedsSaving>();

        let save_time = (Instant::now() - start_time).as_micros();
        diagnostics.add_measurement(&SAVE_TIME_DIAGNOSTIC, || save_time as f64);
        chunk_count += 1;
    }
}

fn sys_load_chunks(
    mut commands: Commands,
    mut diagnostics: Diagnostics,
    mut voxel_world: ResMut<VoxelWorld>,
    query_chunks: Query<(Entity, &Chunk), With<ChunkNeedsLoading>>,
) {
    let mut chunk_count = 0;
    for (id, chunk) in &query_chunks {
        // TODO: Make chunks per frame configurable
        if chunk_count > 8 {
            return;
        }

        let start_time = Instant::now();

        let save_dir = format!("./saves/{}", voxel_world.world_name);
        let path = format!(
            "{}/{}_{}_{}.dat",
            &save_dir, chunk.world_pos[0], chunk.world_pos[1], chunk.world_pos[2]
        );
        let Ok(true) = fs::exists(&path) else {
            commands.entity(id).remove::<ChunkNeedsLoading>();
            commands.entity(id).insert(ChunkNeedsGenerating);
            continue;
        };

        let mut decompressed_buffer = vec![];
        let f = File::open(&path).unwrap();
        let mut decoder = DeflateDecoder::new(f);
        decoder.read_to_end(&mut decompressed_buffer).unwrap();

        let buffer = bitcode::decode(&decompressed_buffer).unwrap();
        match voxel_world.get_chunk_mut(&chunk.world_pos) {
            Some(data) => *data = buffer,
            None => voxel_world.load_chunk(&chunk.world_pos, buffer),
        }

        commands.entity(id).remove::<ChunkNeedsLoading>();
        commands.entity(id).insert(ChunkNeedsMeshing);

        let load_time = (Instant::now() - start_time).as_micros();
        diagnostics.add_measurement(&LOAD_TIME_DIAGNOSTIC, || load_time as f64);
        chunk_count += 1;
    }
}

fn scheduled_save(
    mut commands: Commands,
    time: Res<Time>,
    mut voxel_world: ResMut<VoxelWorld>,
    query_chunks: Query<(Entity, &Chunk), Without<ChunkNeedsSaving>>,
) {
    voxel_world.save_timer.tick(time.delta());
    if !voxel_world.save_timer.finished() {
        return;
    }

    info!("saving...");
    for (id, chunk) in &query_chunks {
        if chunk.dirty {
            commands.entity(id).insert(ChunkNeedsSaving);
        }
    }
}

fn unload_chunks(
    mut commands: Commands,
    mut voxel_world: ResMut<VoxelWorld>,
    query_chunks: Query<(Entity, &Chunk), With<ChunkNeedsUnloading>>,
) {
    for (id, chunk) in &query_chunks {
        voxel_world.unload_chunk(&chunk.world_pos);
        commands.entity(id).despawn();
    }
}
