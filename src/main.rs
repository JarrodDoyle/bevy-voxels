use bevy::{
    asset::RenderAssetUsages,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
    utils::hashbrown::HashMap,
    window::PresentMode,
};
use bevy_flycam::PlayerPlugin;
use fastnoise2::SafeNode;

#[derive(Clone, Copy, PartialEq)]
enum BlockType {
    Grass,
    Dirt,
    Stone,
    Air,
}

#[derive(Resource)]
struct WorldNoise {
    terrain: SafeNode,
}

#[derive(Component)]
struct VoxelStorage {
    chunk_len: usize,
    voxels: HashMap<[i32; 3], Vec<BlockType>>,
}

impl VoxelStorage {
    fn get_voxel(
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

    fn set_voxel(
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

    fn local_pos_to_idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.chunk_len + z * self.chunk_len * self.chunk_len
    }
}

#[derive(Component)]
struct Chunk {
    world_pos: [i32; 3],
}

#[derive(Component)]
struct ChunkNeedsMeshing;

#[derive(Resource)]
struct LoadingTexture {
    is_loaded: bool,
    handle: Handle<Image>,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ArrayTextureMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    array_texture: Handle<Image>,
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

const ATTRIBUTE_TEXTURE: MeshVertexAttribute =
    MeshVertexAttribute::new("texure_id", 988540917, VertexFormat::Uint32);
fn gen_chunk_mesh(world_pos: &[i32; 3], storage: &VoxelStorage) -> Option<Mesh> {
    const NEIGHBOUR_OFFSETS: [[i32; 3]; 6] = [
        [1, 0, 0],  // right
        [-1, 0, 0], // left
        [0, -1, 0], // down
        [0, 1, 0],  // up
        [0, 0, 1],  // forward
        [0, 0, -1], // back
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
        0, 1, 2, 3, 6, 7, 4, 5, 7, 2, 1, 4, 3, 6, 5, 0, 5, 4, 1, 0, 3, 2, 7, 6,
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

    let mut vs: Vec<[f32; 3]> = vec![];
    let mut is = vec![];
    let mut ns = vec![];
    let mut uvs = vec![];
    let mut ts = vec![];
    for z in 0..storage.chunk_len {
        for y in 0..storage.chunk_len {
            for x in 0..storage.chunk_len {
                let block_type = storage.get_voxel(world_pos, x, y, z);
                if block_type.is_none() || block_type.unwrap() == BlockType::Air {
                    continue;
                }

                let xf = x as f32;
                let yf = y as f32;
                let zf = z as f32;

                for i in 0..NEIGHBOUR_OFFSETS.len() {
                    let offset = NEIGHBOUR_OFFSETS[i];
                    let (n_chunk_pos, n_local_x, n_local_y, n_local_z) = get_neighbour_pos(
                        storage.chunk_len,
                        world_pos,
                        &[x as i32, y as i32, z as i32],
                        &offset,
                    );

                    let n_block_type =
                        storage.get_voxel(&n_chunk_pos, n_local_x, n_local_y, n_local_z);
                    if n_block_type.is_none() || n_block_type.unwrap() == BlockType::Air {
                        let vcount = vs.len() as u32;
                        for j in 0..4 {
                            let raw_v = RAW_VERTICES[RAW_INDICES[i * 4 + j]];
                            vs.push([xf + raw_v.0, yf + raw_v.1, zf + raw_v.2]);
                            ns.push([offset[0] as f32, offset[1] as f32, offset[2] as f32]);
                            ts.push(block_type.unwrap() as u32);
                        }

                        uvs.push([0., 0.]);
                        uvs.push([0., 1.]);
                        uvs.push([1., 1.]);
                        uvs.push([1., 0.]);

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
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(ATTRIBUTE_TEXTURE, ts)
        .with_inserted_indices(Indices::U32(is)),
    )
}

fn setup_noise(mut commands: Commands) {
    let encoded_node_tree = "DQADAAAAAAAAQCkAAAAAAD8AAAAAAA==";
    let node = SafeNode::from_encoded_node_tree(encoded_node_tree).unwrap();

    commands.insert_resource(WorldNoise { terrain: node });
}

fn setup_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        handle: asset_server.load("textures/array_texture.png"),
    });
}

fn sys_create_array_texture(
    asset_server: Res<AssetServer>,
    mut loading_texture: ResMut<LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if loading_texture.is_loaded
        || !asset_server
            .load_state(loading_texture.handle.id())
            .is_loaded()
    {
        return;
    }
    loading_texture.is_loaded = true;
    let image = images.get_mut(&loading_texture.handle).unwrap();

    // Create a new array texture asset from the loaded texture.
    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);
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

fn chunk_pos_to_idx(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_LEN + z * CHUNK_LEN * CHUNK_LEN
}

fn sys_chunk_spawner(mut commands: Commands, world_noise: Res<WorldNoise>) {
    let mut chunk_voxel_hashmap = HashMap::<[i32; 3], Vec<BlockType>>::new();

    let mut noise_vals = vec![0.0; CHUNK_LEN * CHUNK_LEN * CHUNK_LEN];

    for z in 0..3 {
        for y in 0..3 {
            for x in 0..3 {
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

                for z in 0..CHUNK_LEN {
                    for y in 0..CHUNK_LEN {
                        for x in 0..CHUNK_LEN {
                            let i = chunk_pos_to_idx(x, y, z);
                            if voxels[i] == BlockType::Air {
                                continue;
                            }

                            for dy in 1..4 {
                                if y + dy < CHUNK_LEN
                                    && voxels[chunk_pos_to_idx(x, y + dy, z)] == BlockType::Air
                                {
                                    voxels[i] = if dy == 1 {
                                        BlockType::Grass
                                    } else {
                                        BlockType::Dirt
                                    };
                                    break;
                                }
                            }
                        }
                    }
                }

                let world_pos = [x as i32, y as i32, z as i32];
                chunk_voxel_hashmap.insert(world_pos, voxels);

                commands
                    .spawn((
                        Chunk { world_pos },
                        ChunkNeedsMeshing,
                        Transform::from_xyz(x as f32 * 32., y as f32 * 32., z as f32 * 32.),
                    ))
                    .observe(break_place_block);
            }
        }
    }

    commands.spawn(VoxelStorage {
        chunk_len: 32,
        voxels: chunk_voxel_hashmap,
    });
}

fn sys_chunk_mesher(
    mut commands: Commands,
    loading_texture: ResMut<LoadingTexture>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
    query_storage: Query<&VoxelStorage>,
    chunks_query: Query<(Entity, &Chunk, &ChunkNeedsMeshing)>,
) {
    if !loading_texture.is_loaded {
        return;
    }

    let voxel_storage = query_storage.single();

    let _colors = [
        Color::srgb_u8(228, 59, 68),
        Color::srgb_u8(62, 137, 72),
        Color::srgb_u8(0, 153, 219),
        Color::srgb_u8(192, 203, 220),
        Color::srgb_u8(254, 231, 97),
        Color::srgb_u8(104, 56, 108),
    ];

    let material_handle = materials.add(ArrayTextureMaterial {
        array_texture: loading_texture.handle.clone(),
    });

    for (id, chunk, _) in &chunks_query {
        if let Some(mesh) = gen_chunk_mesh(&chunk.world_pos, voxel_storage) {
            commands.entity(id).remove::<ChunkNeedsMeshing>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material_handle.clone()),
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
    mut query_storage: Query<&mut VoxelStorage>,
    query_chunk: Query<(Entity, &Chunk)>,
) {
    let mut storage = query_storage.single_mut();

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Voxel Meshing".into(),
                        name: Some("bevy.app".into()),
                        resolution: (1280., 720.).into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            MeshPickingPlugin,
            MaterialPlugin::<ArrayTextureMaterial>::default(),
        ))
        .add_plugins(PlayerPlugin)
        .add_systems(
            Startup,
            (setup_assets, setup_noise, sys_chunk_spawner).chain(),
        )
        .add_systems(
            Update,
            (toggle_vsync, sys_chunk_mesher, sys_create_array_texture),
        )
        .run();
}
