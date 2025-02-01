use bevy::{
    asset::RenderAssetUsages,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    window::PresentMode,
};
use bevy_flycam::PlayerPlugin;

#[derive(Clone, Copy, PartialEq)]
enum BlockType {
    Stone,
    Air,
}

#[derive(Component)]
struct Chunk;

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
    ));
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
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    const CHUNK_LEN: usize = 32;
    let mut voxels = [BlockType::Air; CHUNK_LEN * CHUNK_LEN * CHUNK_LEN];
    (0..CHUNK_LEN * CHUNK_LEN * CHUNK_LEN).for_each(|i| {
        if i % 3 == 0 {
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

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, ns)
    .with_inserted_indices(Indices::U32(is));

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb_u8(200, 100, 90))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
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
        .add_systems(Startup, setup_temp_geometry)
        .add_systems(Update, toggle_vsync)
        .run();
}
