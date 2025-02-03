use bevy::{pbr::wireframe::Wireframe, prelude::*};

use crate::{asset_registry::AssetRegistry, screens::Screen};

use super::chunk::{Chunk, ChunkNeedsMeshing, VoxelStorage};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup_hover);
}

fn setup_hover(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        HoverHighlight,
        Mesh3d(meshes.add(Cuboid::default())),
        Transform::default(),
        Visibility::Hidden,
        Wireframe,
        PickingBehavior::IGNORE,
    ));
}

#[derive(Component)]
pub struct HoverHighlight;

pub fn hover_block(
    _trigger: Trigger<Pointer<Over>>,
    mut query: Query<&mut Visibility, With<HoverHighlight>>,
) {
    *query.single_mut() = Visibility::Visible;
}

pub fn hover_move_block(
    trigger: Trigger<Pointer<Move>>,
    mut query: Query<&mut Transform, With<HoverHighlight>>,
) {
    let hit_pos = (trigger.hit.position.unwrap() - trigger.hit.normal.unwrap() * 0.01).floor();
    query.single_mut().translation = hit_pos + Vec3::new(0.5, 0.5, 0.5);
}

pub fn unhover_block(
    _trigger: Trigger<Pointer<Out>>,
    mut query: Query<&mut Visibility, With<HoverHighlight>>,
) {
    *query.single_mut() = Visibility::Hidden;
}

pub fn break_place_block(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    registry: Res<AssetRegistry>,
    mut query_storage: Query<&mut VoxelStorage>,
    query_chunk: Query<(Entity, &Chunk)>,
) {
    let mut storage = query_storage.single_mut();

    let hit_pos = (click.hit.position.unwrap() - click.hit.normal.unwrap() * 0.01).floor();
    let (world_pos, block_type) = match click.button {
        PointerButton::Primary => (hit_pos, registry.get_block_id("air")),
        PointerButton::Secondary => (
            (hit_pos + click.hit.normal.unwrap()).floor(),
            registry.get_block_id("stone"),
        ),
        PointerButton::Middle => return,
    };

    let cx = (world_pos[0] / storage.chunk_len as f32).floor() as i32;
    let cy = (world_pos[1] / storage.chunk_len as f32).floor() as i32;
    let cz = (world_pos[2] / storage.chunk_len as f32).floor() as i32;
    let local_x = (world_pos[0] as i32 - cx * storage.chunk_len as i32) as usize;
    let local_y = (world_pos[1] as i32 - cy * storage.chunk_len as i32) as usize;
    let local_z = (world_pos[2] as i32 - cz * storage.chunk_len as i32) as usize;

    storage.set_voxel(&[cx, cy, cz], local_x, local_y, local_z, block_type);

    let mut needs_meshing = vec![[cx, cy, cz]];
    if local_x == 0 {
        needs_meshing.push([cx - 1, cy, cz]);
    }
    if local_x == storage.chunk_len - 1 {
        needs_meshing.push([cx + 1, cy, cz]);
    }
    if local_y == 0 {
        needs_meshing.push([cx, cy - 1, cz]);
    }
    if local_y == storage.chunk_len - 1 {
        needs_meshing.push([cx, cy + 1, cz]);
    }
    if local_z == 0 {
        needs_meshing.push([cx, cy, cz - 1]);
    }
    if local_z == storage.chunk_len - 1 {
        needs_meshing.push([cx, cy, cz + 1]);
    }

    for (id, chunk) in &query_chunk {
        if needs_meshing.contains(&chunk.world_pos) {
            commands.entity(id).insert(ChunkNeedsMeshing);
        }
    }
}
