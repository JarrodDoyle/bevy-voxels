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

fn gen_chunk_mesh(x: usize, y: usize, z: usize, world_noise: &Res<WorldNoise>) -> Mesh {
    const CHUNK_LEN: usize = 32;
    const FREQUENCY: f32 = 0.005;
    const SEED: i32 = 1338;

    let mut noise_vals = vec![0.0; CHUNK_LEN * CHUNK_LEN * CHUNK_LEN];
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

    let mut voxels = [BlockType::Air; CHUNK_LEN * CHUNK_LEN * CHUNK_LEN];
    (0..CHUNK_LEN * CHUNK_LEN * CHUNK_LEN).for_each(|i| {
        if noise_vals[i] > 0. {
            voxels[i] = BlockType::Stone;
        }
    });

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

                let i_offset = vs.len() as u32;

                // top
                vs.push([xf, yf + 1., zf]);
                vs.push([xf, yf + 1., zf + 1.]);
                vs.push([xf + 1., yf + 1., zf + 1.]);
                vs.push([xf + 1., yf + 1., zf]);
                ns.push([0., 1., 0.]);
                ns.push([0., 1., 0.]);
                ns.push([0., 1., 0.]);
                ns.push([0., 1., 0.]);
                is.push(i_offset);
                is.push(i_offset + 1);
                is.push(i_offset + 2);
                is.push(i_offset);
                is.push(i_offset + 2);
                is.push(i_offset + 3);

                // bottom
                vs.push([xf, yf, zf]);
                vs.push([xf + 1., yf, zf]);
                vs.push([xf + 1., yf, zf + 1.]);
                vs.push([xf, yf, zf + 1.]);
                ns.push([0., -1., 0.]);
                ns.push([0., -1., 0.]);
                ns.push([0., -1., 0.]);
                ns.push([0., -1., 0.]);
                is.push(i_offset + 4);
                is.push(i_offset + 5);
                is.push(i_offset + 6);
                is.push(i_offset + 4);
                is.push(i_offset + 6);
                is.push(i_offset + 7);

                // front
                vs.push([xf, yf, zf]);
                vs.push([xf, yf + 1., zf]);
                vs.push([xf + 1., yf + 1., zf]);
                vs.push([xf + 1., yf, zf]);
                ns.push([-1., 0., 0.]);
                ns.push([-1., 0., 0.]);
                ns.push([-1., 0., 0.]);
                ns.push([-1., 0., 0.]);
                is.push(i_offset + 8);
                is.push(i_offset + 9);
                is.push(i_offset + 10);
                is.push(i_offset + 8);
                is.push(i_offset + 10);
                is.push(i_offset + 11);

                // back
                vs.push([xf, yf, zf + 1.]);
                vs.push([xf + 1., yf, zf + 1.]);
                vs.push([xf + 1., yf + 1., zf + 1.]);
                vs.push([xf, yf + 1., zf + 1.]);
                ns.push([1., 0., 0.]);
                ns.push([1., 0., 0.]);
                ns.push([1., 0., 0.]);
                ns.push([1., 0., 0.]);
                is.push(i_offset + 12);
                is.push(i_offset + 13);
                is.push(i_offset + 14);
                is.push(i_offset + 12);
                is.push(i_offset + 14);
                is.push(i_offset + 15);

                // left
                vs.push([xf, yf, zf + 1.]);
                vs.push([xf, yf + 1., zf + 1.]);
                vs.push([xf, yf + 1., zf]);
                vs.push([xf, yf, zf]);
                ns.push([0., 0., -1.]);
                ns.push([0., 0., -1.]);
                ns.push([0., 0., -1.]);
                ns.push([0., 0., -1.]);
                is.push(i_offset + 16);
                is.push(i_offset + 17);
                is.push(i_offset + 18);
                is.push(i_offset + 16);
                is.push(i_offset + 18);
                is.push(i_offset + 19);

                // right
                vs.push([xf + 1., yf, zf + 1.]);
                vs.push([xf + 1., yf, zf]);
                vs.push([xf + 1., yf + 1., zf]);
                vs.push([xf + 1., yf + 1., zf + 1.]);
                ns.push([0., 0., 1.]);
                ns.push([0., 0., 1.]);
                ns.push([0., 0., 1.]);
                ns.push([0., 0., 1.]);
                is.push(i_offset + 20);
                is.push(i_offset + 21);
                is.push(i_offset + 22);
                is.push(i_offset + 20);
                is.push(i_offset + 22);
                is.push(i_offset + 23);
            }
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, ns)
    .with_inserted_indices(Indices::U32(is))
}

fn setup_temp_geometry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    noise: Res<WorldNoise>,
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

    let colors = [
        Color::srgb_u8(228, 59, 68),
        Color::srgb_u8(62, 137, 72),
        Color::srgb_u8(0, 153, 219),
        Color::srgb_u8(192, 203, 220),
        Color::srgb_u8(254, 231, 97),
        Color::srgb_u8(104, 56, 108),
    ];

    let mut i = 0;
    for z in 0..5 {
        for y in 0..5 {
            for x in 0..5 {
                commands.spawn((
                    Mesh3d(meshes.add(gen_chunk_mesh(x, y, z, &noise))),
                    MeshMaterial3d(materials.add(colors[i % colors.len()])),
                    Transform::from_xyz(x as f32 * 32., y as f32 * 32., z as f32 * 32.),
                ));
                i += 1;
            }
        }
    }
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
        ))
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, (setup_noise, setup_temp_geometry).chain())
        .add_systems(Update, toggle_vsync)
        .run();
}
