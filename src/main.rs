mod assets;
mod block_type;
mod model;

use assets::{BlockArrayTextureHandle, BlockAssets, BlockTextureIds, ModelAssets, TextureAssets};
use bevy::{
    asset::RenderAssetUsages,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, VertexFormat,
        },
    },
    utils::hashbrown::HashMap,
    window::PresentMode,
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_flycam::PlayerPlugin;
use block_type::{Block, BlockType};
use fastnoise2::SafeNode;
use model::Model;

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

    fn load_chunk(&mut self, chunk_pos: &[i32; 3], chunk_voxels: Vec<BlockType>) {
        assert_eq!(
            chunk_voxels.len(),
            self.chunk_len * self.chunk_len * self.chunk_len
        );

        self.voxels.insert(*chunk_pos, chunk_voxels);
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

fn setup_noise(mut commands: Commands) {
    let encoded_node_tree = "DQADAAAAAAAAQCkAAAAAAD8AAAAAAA==";
    let node = SafeNode::from_encoded_node_tree(encoded_node_tree).unwrap();

    commands.insert_resource(WorldNoise { terrain: node });
}

fn sys_create_array_texture(
    mut commands: Commands,
    block_array_texture: Option<ResMut<BlockArrayTextureHandle>>,
    asset_server: Res<AssetServer>,
    textures: Res<TextureAssets>,
    images: ResMut<Assets<Image>>,
) {
    if block_array_texture.is_some() {
        return;
    }

    const SIZE: u32 = 16;
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE,
            height: SIZE * textures.blocks.len() as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let mut id_map = HashMap::<String, u32>::new();
    for (z, k) in textures.blocks.keys().enumerate() {
        id_map.insert(k.0.clone(), z as u32);

        let t = images.get(textures.blocks[k].id()).unwrap();
        for y in 0..SIZE {
            for x in 0..SIZE {
                let c = t.get_color_at(x, y).unwrap();
                let _ = image.set_color_at(x, y + z as u32 * SIZE, c);
            }
        }
    }

    image.reinterpret_stacked_2d_as_array(textures.blocks.len() as u32);

    let handle = asset_server.add(image);
    commands.insert_resource(BlockArrayTextureHandle(handle));
    commands.insert_resource(BlockTextureIds(id_map));
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

fn sys_chunk_spawner(
    mut commands: Commands,
    world_noise: Res<WorldNoise>,
    blocks: Res<BlockAssets>,
    block_handles: Res<Assets<Block>>,
) {
    let mut storage = VoxelStorage {
        chunk_len: 32,
        voxels: HashMap::<[i32; 3], Vec<BlockType>>::new(),
    };

    let mut block_map = HashMap::<String, u32>::new();
    for i in 0..blocks.folder.len() {
        let b = block_handles.get(blocks.folder[i].id()).unwrap();
        block_map.insert(b.identifier.clone(), i as u32);
    }

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

                let mut chunk_voxels = vec![block_map["air"]; voxels_per_chunk];
                (0..voxels_per_chunk).for_each(|i| {
                    if noise_vals[i] > 0. {
                        chunk_voxels[i] = block_map["stone"];
                    }
                });

                for z in 0..storage.chunk_len {
                    for y in 0..storage.chunk_len {
                        for x in 0..storage.chunk_len {
                            let i = storage.local_pos_to_idx(x, y, z);
                            if chunk_voxels[i] == block_map["air"] {
                                continue;
                            }

                            for dy in 1..4 {
                                if y + dy < storage.chunk_len
                                    && chunk_voxels[storage.local_pos_to_idx(x, y + dy, z)]
                                        == block_map["air"]
                                {
                                    chunk_voxels[i] = if dy == 1 {
                                        block_map["grass"]
                                    } else {
                                        block_map["dirt"]
                                    };
                                    break;
                                }
                            }
                        }
                    }
                }

                let world_pos = [x as i32, y as i32, z as i32];
                storage.load_chunk(&world_pos, chunk_voxels);

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

    commands.spawn(storage);
}

fn sys_chunk_mesher(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    block_array_texture: Res<BlockArrayTextureHandle>,
    block_texture_ids: Res<BlockTextureIds>,
    model_handle: Res<ModelAssets>,
    res_model: Res<Assets<Model>>,
    blocks: Res<BlockAssets>,
    block_assets: Res<Assets<Block>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
    query_storage: Query<&VoxelStorage>,
    chunks_query: Query<(Entity, &Chunk, &ChunkNeedsMeshing)>,
) {
    if !asset_server.is_loaded(&block_array_texture.0) {
        return;
    }

    let mut model_map = HashMap::<String, u32>::new();
    for i in 0..model_handle.folder.len() {
        let m = res_model.get(model_handle.folder[i].id()).unwrap();
        model_map.insert(m.identifier.clone(), i as u32);
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
        array_texture: block_array_texture.0.clone(),
    });

    for (id, chunk, _) in &chunks_query {
        let world_pos: &[i32; 3] = &chunk.world_pos;
        let block_texture_ids: &BlockTextureIds = &block_texture_ids;
        let blocks: &BlockAssets = &blocks;
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

        let mut vs: Vec<[f32; 3]> = vec![];
        let mut is = vec![];
        let mut ns = vec![];
        let mut uvs = vec![];
        let mut ts = vec![];
        for z in 0..voxel_storage.chunk_len {
            for y in 0..voxel_storage.chunk_len {
                for x in 0..voxel_storage.chunk_len {
                    let block = match voxel_storage.get_voxel(world_pos, x, y, z) {
                        Some(i) => block_assets.get(blocks.folder[i as usize].id()),
                        None => None,
                    };

                    if block.is_none_or(|b| b.model.is_none()) {
                        continue;
                    }

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
                            Some(i) => block_assets.get(blocks.folder[i as usize].id()),
                            None => None,
                        };

                        cull[i] = n_block.is_some_and(|b| b.model == block.unwrap().model);
                    }

                    let xf = x as f32;
                    let yf = y as f32;
                    let zf = z as f32;

                    let mut texture_map = HashMap::<String, u32>::new();
                    let textures = &block.unwrap().textures;
                    for k in textures.keys() {
                        texture_map.insert(k.to_string(), block_texture_ids.0[&textures[k]]);
                    }

                    let model_name = &block.unwrap().model.clone().unwrap();
                    let model = res_model
                        .get(model_handle.folder[model_map[model_name] as usize].id())
                        .unwrap();

                    model.mesh(
                        &cull,
                        &[xf, yf, zf],
                        &mut vs,
                        &mut ns,
                        &mut uvs,
                        &mut ts,
                        &mut is,
                        &texture_map,
                    );
                }
            }
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
            0,
        ),
        PointerButton::Secondary => (
            (click.hit.position.unwrap() + click.hit.normal.unwrap() * 0.01).floor(),
            1,
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum States {
    #[default]
    AssetLoading,
    Loaded,
    Running,
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
            RonAssetPlugin::<Model>::new(&["model.ron"]),
            RonAssetPlugin::<Block>::new(&["block.ron"]),
        ))
        .add_plugins(PlayerPlugin)
        .init_state::<States>()
        .add_loading_state(
            LoadingState::new(States::AssetLoading)
                .continue_to_state(States::Loaded)
                .load_collection::<ModelAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<BlockAssets>(),
        )
        .add_loading_state(LoadingState::new(States::Loaded).continue_to_state(States::Running))
        .add_systems(
            OnEnter(States::Loaded),
            (sys_chunk_spawner, sys_create_array_texture),
        )
        .add_systems(Startup, (setup_noise,).chain())
        .add_systems(
            Update,
            (
                toggle_vsync,
                (sys_chunk_mesher.run_if(in_state(States::Running)),).chain(),
            ),
        )
        .run();
}
