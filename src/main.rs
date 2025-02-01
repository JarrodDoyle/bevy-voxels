use bevy::{
    asset::RenderAssetUsages,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::PresentMode,
};
use bevy_flycam::PlayerPlugin;
use fastnoise2::SafeNode;

#[derive(Clone, Copy, PartialEq)]
enum BlockType {
    Stone,
    Air,
}

#[derive(Resource)]
struct WorldNoise {
    terrain: SafeNode,
}

#[derive(Component)]
struct Chunk {
    world_pos: [i32; 3],
    voxels: Vec<BlockType>,
}

#[derive(Component)]
struct ChunkNeedsMeshing;

fn gen_chunk_mesh(voxels: &[BlockType]) -> Option<Mesh> {
    const NEIGHBOUR_OFFSETS: [(i32, i32, i32); 6] = [
        (1, 0, 0),  // right
        (-1, 0, 0), // left
        (0, -1, 0), // down
        (0, 1, 0),  // up
        (0, 0, 1),  // forward
        (0, 0, -1), // back
    ];
    const RAW_VERTICES: [(f32, f32, f32); 8] = [
        (1., 1., 1.),
        (1., 0., 1.),
        (1., 0., 0.),
        (1., 1., 0.),
        (0., 0., 1.),
        (0., 1., 1.),
        (0., 1., 0.),
        (0., 0., 0.),
    ];
    const RAW_INDICES: [usize; 24] = [
        0, 1, 2, 3, 4, 5, 6, 7, 1, 4, 7, 2, 5, 0, 3, 6, 5, 4, 1, 0, 3, 2, 7, 6,
    ];

    let mut vs: Vec<[f32; 3]> = vec![];
    let mut is = vec![];
    let mut ns = vec![];
    for z in 0..CHUNK_LEN {
        for y in 0..CHUNK_LEN {
            for x in 0..CHUNK_LEN {
                let idx = x + y * CHUNK_LEN + z * CHUNK_LEN * CHUNK_LEN;
                if voxels[idx] == BlockType::Air {
                    continue;
                }

                let xf = x as f32;
                let yf = y as f32;
                let zf = z as f32;

                for i in 0..NEIGHBOUR_OFFSETS.len() {
                    let offset = NEIGHBOUR_OFFSETS[i];
                    let nx = (x as i32) + offset.0;
                    let ny = (y as i32) + offset.1;
                    let nz = (z as i32) + offset.2;
                    let nidx =
                        nx + ny * CHUNK_LEN as i32 + nz * CHUNK_LEN as i32 * CHUNK_LEN as i32;

                    if nx < 0
                        || nx >= CHUNK_LEN as i32
                        || ny < 0
                        || ny >= CHUNK_LEN as i32
                        || nz < 0
                        || nz >= CHUNK_LEN as i32
                        || voxels[nidx as usize] == BlockType::Air
                    {
                        let vcount = vs.len() as u32;
                        for j in 0..4 {
                            let raw_v = RAW_VERTICES[RAW_INDICES[i * 4 + j]];
                            vs.push([xf + raw_v.0, yf + raw_v.1, zf + raw_v.2]);
                            ns.push([offset.0 as f32, offset.1 as f32, offset.2 as f32]);
                        }
                        is.push(vcount);
                        is.push(vcount + 2);
                        is.push(vcount + 3);
                        is.push(vcount);
                        is.push(vcount + 1);
                        is.push(vcount + 2);
                    }
                }
            }
        }
    }

    if vs.is_empty() {
        return None;
    }

    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, ns)
        .with_inserted_indices(Indices::U32(is)),
    )
}

fn setup_temp_geometry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8., 8.0, 4.0),
    ));
}

fn setup_noise(mut commands: Commands) {
    let encoded_node_tree = "DQADAAAAAAAAQCkAAAAAAD8AAAAAAA==";
    let node = SafeNode::from_encoded_node_tree(encoded_node_tree).unwrap();

    commands.insert_resource(WorldNoise { terrain: node });
}

fn toggle_vsync(input: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>) {
    if input.just_pressed(KeyCode::KeyV) {
        let mut window = windows.single_mut();

        window.present_mode = if matches!(window.present_mode, PresentMode::AutoVsync) {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };
        info!("PRESENT_MODE: {:?}", window.present_mode);
    }
}

const CHUNK_LEN: usize = 32;
const FREQUENCY: f32 = 0.005;
const SEED: i32 = 1338;

fn sys_chunk_spawner(mut commands: Commands, world_noise: Res<WorldNoise>) {
    let mut noise_vals = vec![0.0; CHUNK_LEN * CHUNK_LEN * CHUNK_LEN];

    for z in 0..9 {
        for y in 0..9 {
            for x in 0..9 {
                world_noise.terrain.gen_uniform_grid_3d(
                    &mut noise_vals,
                    (CHUNK_LEN * x) as i32,
                    (CHUNK_LEN * y) as i32,
                    (CHUNK_LEN * z) as i32,
                    CHUNK_LEN as i32,
                    CHUNK_LEN as i32,
                    CHUNK_LEN as i32,
                    FREQUENCY,
                    SEED,
                );

                let mut voxels = vec![BlockType::Air; CHUNK_LEN * CHUNK_LEN * CHUNK_LEN];
                (0..CHUNK_LEN * CHUNK_LEN * CHUNK_LEN).for_each(|i| {
                    if noise_vals[i] > 0. {
                        voxels[i] = BlockType::Stone;
                    }
                });

                commands
                    .spawn((
                        Chunk {
                            world_pos: [x as i32, y as i32, z as i32],
                            voxels,
                        },
                        ChunkNeedsMeshing,
                        Transform::from_xyz(x as f32 * 32., y as f32 * 32., z as f32 * 32.),
                    ))
                    .observe(break_place_block);
            }
        }
    }
}

fn sys_chunk_mesher(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks_query: Query<(Entity, &Chunk, &ChunkNeedsMeshing)>,
) {
    let colors = [
        Color::srgb_u8(228, 59, 68),
        Color::srgb_u8(62, 137, 72),
        Color::srgb_u8(0, 153, 219),
        Color::srgb_u8(192, 203, 220),
        Color::srgb_u8(254, 231, 97),
        Color::srgb_u8(104, 56, 108),
    ];

    for (id, chunk, _) in &chunks_query {
        if let Some(mesh) = gen_chunk_mesh(&chunk.voxels) {
            commands.entity(id).remove::<ChunkNeedsMeshing>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(colors[id.index() as usize % colors.len()])),
            ));
        } else {
            // TODO: Remove meshmaterial?
            commands.entity(id).remove::<(Mesh3d, ChunkNeedsMeshing)>();
        }
    }
}

fn break_place_block(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Chunk)>,
) {
    let (world_pos, block_type) = match click.button {
        PointerButton::Primary => (
            (click.hit.position.unwrap() - click.hit.normal.unwrap() * 0.01).floor(),
            BlockType::Air,
        ),
        PointerButton::Secondary => (
            (click.hit.position.unwrap() + click.hit.normal.unwrap() * 0.01).floor(),
            BlockType::Stone,
        ),
        PointerButton::Middle => return,
    };

    let cx = (world_pos[0] / CHUNK_LEN as f32).floor() as i32;
    let cy = (world_pos[1] / CHUNK_LEN as f32).floor() as i32;
    let cz = (world_pos[2] / CHUNK_LEN as f32).floor() as i32;

    let local_x = (world_pos[0] as i32 - cx * CHUNK_LEN as i32) as usize;
    let local_y = (world_pos[1] as i32 - cy * CHUNK_LEN as i32) as usize;
    let local_z = (world_pos[2] as i32 - cz * CHUNK_LEN as i32) as usize;

    for (id, mut chunk) in &mut query {
        if cx == chunk.world_pos[0] && cy == chunk.world_pos[1] && cz == chunk.world_pos[2] {
            let idx = local_x + local_y * CHUNK_LEN + local_z * CHUNK_LEN * CHUNK_LEN;
            chunk.voxels[idx] = block_type;
            commands.entity(id).insert(ChunkNeedsMeshing);
            break;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Voxel Meshing".into(),
                    name: Some("bevy.app".into()),
                    resolution: (1280., 720.).into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            MeshPickingPlugin,
        ))
        .add_plugins(PlayerPlugin)
        .add_systems(
            Startup,
            (setup_noise, setup_temp_geometry, sys_chunk_spawner).chain(),
        )
        .add_systems(Update, (toggle_vsync, sys_chunk_mesher))
        .run();
}
