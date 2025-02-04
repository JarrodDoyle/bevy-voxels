use bevy::{
    input::mouse::MouseMotion,
    pbr::wireframe::Wireframe,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{asset_registry::AssetRegistry, screens::Screen};

use super::chunk::{Chunk, ChunkNeedsMeshing, VoxelStorage};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(PlayerMovementControls {
        toggle_control: KeyCode::Escape,
        left: KeyCode::KeyA,
        right: KeyCode::KeyD,
        up: KeyCode::Space,
        down: KeyCode::ShiftLeft,
        forward: KeyCode::KeyW,
        backward: KeyCode::KeyS,
        mouse_sensitivity: 0.0001,
    });
    app.add_systems(OnEnter(Screen::Gameplay), (setup_hover, setup_player));
    app.add_systems(
        Update,
        (player_move, player_look, player_toggle_active).run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource)]
pub struct PlayerMovementControls {
    pub toggle_control: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub forward: KeyCode,
    pub backward: KeyCode,
    pub mouse_sensitivity: f32,
}

#[derive(Component)]
pub struct MovementSettings {
    base_speed: f32,
    speed: f32,
    active: bool,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            base_speed: 10.,
            speed: 10.,
            active: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct Player;

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Transform::default(),
        Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Player,
        MovementSettings::default(),
    ));
}

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    controls: Res<PlayerMovementControls>,
    mut query_player: Query<(&mut Transform, &MovementSettings), With<Player>>,
) {
    let (mut transform, movement_settings) = query_player.single_mut();
    if !movement_settings.active {
        return;
    }

    let local_z = transform.local_z();
    let forward = -Vec3::new(local_z.x, 0., local_z.z);
    let right = Vec3::new(local_z.z, 0., -local_z.x);

    let mut target_velocity = Vec3::ZERO;
    for key in keys.get_pressed() {
        let k = *key;
        target_velocity += if k == controls.forward {
            forward
        } else if k == controls.backward {
            -forward
        } else if k == controls.left {
            -right
        } else if k == controls.right {
            right
        } else if k == controls.up {
            Vec3::Y
        } else if k == controls.down {
            -Vec3::Y
        } else {
            Vec3::ZERO
        };
    }

    target_velocity = target_velocity.normalize_or_zero();
    transform.translation += target_velocity * time.delta_secs() * movement_settings.speed;
}

fn player_look(
    controls: Res<PlayerMovementControls>,
    mut mouse_motions: EventReader<MouseMotion>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    mut query_player: Query<(&mut Transform, &MovementSettings), With<Player>>,
) {
    let window = query_window.single();
    let (mut transform, movement_settings) = query_player.single_mut();
    if !movement_settings.active {
        return;
    }

    let sensitivity = controls.mouse_sensitivity;
    let (mut target_yaw, mut target_pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    for event in mouse_motions.read() {
        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
        let window_scale = window.height().min(window.width());
        target_pitch -= (sensitivity * event.delta.y * window_scale).to_radians();
        target_yaw -= (sensitivity * event.delta.x * window_scale).to_radians();
    }

    let yaw_rot = Quat::from_axis_angle(Vec3::Y, target_yaw);
    let pitch_rot = Quat::from_axis_angle(Vec3::X, target_pitch.clamp(-1.54, 1.54));
    transform.rotation = yaw_rot * pitch_rot;
}

fn player_toggle_active(
    controls: Res<PlayerMovementControls>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query_window: Query<&mut Window, With<PrimaryWindow>>,
    mut query_player: Query<&mut MovementSettings, With<Player>>,
) {
    if !keys.just_pressed(controls.toggle_control) {
        return;
    }

    let mut movement_settings = query_player.single_mut();
    movement_settings.active = !movement_settings.active;

    let mut window = query_window.single_mut();
    match movement_settings.active {
        true => {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
        false => {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    };
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
