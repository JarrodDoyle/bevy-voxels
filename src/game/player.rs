use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    pbr::wireframe::Wireframe,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    assets::{BlockType, Registry},
    render::ChunkNeedsMeshing,
    screens::Screen,
};

use super::chunk::{Chunk, VoxelStorage};

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
        decrease_speed: KeyCode::Minus,
        increase_speed: KeyCode::Equal,
    });
    app.add_systems(OnEnter(Screen::Gameplay), setup_player);
    app.add_systems(
        Update,
        (
            player_move,
            player_look,
            player_toggle_active,
            player_show_block_highlight,
            player_break_place_block,
            player_modify_speed,
            player_scroll_inventory,
        )
            .run_if(in_state(Screen::Gameplay)),
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
    pub increase_speed: KeyCode,
    pub decrease_speed: KeyCode,
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

#[derive(Component)]
pub struct HoverHighlight;

#[derive(Component, Default)]
pub struct TargetBlock {
    pub chunk_pos: [i32; 3],
    pub local_pos: [usize; 3],
    pub block_id: usize,
    pub model_id: Option<usize>,
}

#[derive(Component)]
pub struct Hotbar {
    pub slots: Vec<Option<BlockType>>,
    pub active_slot: usize,
}

fn setup_player(mut commands: Commands, registry: Res<Registry>, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        StateScoped(Screen::Gameplay),
        Camera3d::default(),
        // Transform::default(),
        Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Player,
        MovementSettings::default(),
        Hotbar {
            slots: vec![
                Some(registry.get_block_id("grass")),
                Some(registry.get_block_id("dirt")),
                Some(registry.get_block_id("stone")),
                Some(registry.get_block_id("stone_fence")),
            ],
            active_slot: 0,
        },
    ));

    commands.spawn((
        HoverHighlight,
        TargetBlock::default(),
        Mesh3d(meshes.add(Cuboid::default())),
        Transform::default(),
        Visibility::Hidden,
        Wireframe,
        PickingBehavior::IGNORE,
        StateScoped(Screen::Gameplay),
    ));
}

fn player_modify_speed(
    keys: Res<ButtonInput<KeyCode>>,
    controls: Res<PlayerMovementControls>,
    mut query_player: Query<&mut MovementSettings, With<Player>>,
) {
    let mut movement_settings = query_player.single_mut();
    if !movement_settings.active {
        return;
    }

    let base = movement_settings.base_speed;
    let mut target_speed = movement_settings.speed;

    if keys.just_pressed(controls.increase_speed) {
        target_speed += base;
    }
    if keys.just_pressed(controls.decrease_speed) {
        target_speed -= base;
    }

    movement_settings.speed = f32::clamp(target_speed, base, base * 10.);
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

fn player_scroll_inventory(
    mut scrolls: EventReader<MouseWheel>,
    mut query_player: Query<(&MovementSettings, &mut Hotbar), With<Player>>,
) {
    let (movement_settings, mut hotbar) = query_player.single_mut();
    if !movement_settings.active {
        return;
    }

    let mut delta = 0.;
    for event in scrolls.read() {
        delta += event.y;
    }
    delta = delta.floor();

    if delta == 0.0 {
        return;
    }

    if delta.is_sign_positive() {
        hotbar.active_slot = (hotbar.active_slot + 1) % hotbar.slots.len();
    } else if delta.is_sign_negative() {
        hotbar.active_slot = (hotbar.active_slot + hotbar.slots.len() - 1) % hotbar.slots.len();
    }
}

fn player_show_block_highlight(
    mut ray_cast: MeshRayCast,
    registry: Res<Registry>,
    storage: Res<VoxelStorage>,
    query_player: Query<&Transform, With<Player>>,
    query_chunk: Query<&Chunk>,
    mut query_highlight: Query<
        (&mut Transform, &mut Visibility, &mut TargetBlock),
        (With<HoverHighlight>, Without<Player>),
    >,
) {
    let player_transform = query_player.single();
    let (mut highlight_transform, mut highlight_visible, mut highlight_target) =
        query_highlight.single_mut();

    let ray = Ray3d::new(player_transform.translation, player_transform.forward());
    let filter = |id| query_chunk.contains(id);
    let raycast_setings = RayCastSettings::default().with_filter(&filter);

    if let Some((_, hit)) = ray_cast.cast_ray(ray, &raycast_setings).first() {
        let hit_pos = (hit.point - hit.normal.normalize_or_zero() * 0.01).floor();
        highlight_transform.translation = hit_pos;
        *highlight_visible = Visibility::Visible;

        let cx = (hit_pos[0] / storage.chunk_len as f32).floor() as i32;
        let cy = (hit_pos[1] / storage.chunk_len as f32).floor() as i32;
        let cz = (hit_pos[2] / storage.chunk_len as f32).floor() as i32;
        let local_x = (hit_pos[0] as i32 - cx * storage.chunk_len as i32) as usize;
        let local_y = (hit_pos[1] as i32 - cy * storage.chunk_len as i32) as usize;
        let local_z = (hit_pos[2] as i32 - cz * storage.chunk_len as i32) as usize;

        // Avoids triggering a Change<TargetBlock>
        let chunk_pos = [cx, cy, cz];
        let local_pos = [local_x, local_y, local_z];
        if highlight_target.chunk_pos == chunk_pos && highlight_target.local_pos == local_pos {
            return;
        }

        if let Some(block_id) = storage.get_voxel(&chunk_pos, local_x, local_y, local_z) {
            let block = registry.get_block(block_id);
            highlight_target.chunk_pos = chunk_pos;
            highlight_target.local_pos = local_pos;
            highlight_target.block_id = block_id;
            highlight_target.model_id = block.model;
        }
    } else {
        *highlight_visible = Visibility::Hidden;
    }
}

// TODO: Refactor to use TargetBlock
pub fn player_break_place_block(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    registry: Res<Registry>,
    mut storage: ResMut<VoxelStorage>,
    mut ray_cast: MeshRayCast,
    query_player: Query<(&Hotbar, &Transform), With<Player>>,
    query_chunk: Query<(Entity, &Chunk)>,
) {
    let (player_hotbar, player_transform) = query_player.single();

    let (normal_multiplier, block_type, destroying) =
        if mouse_buttons.just_pressed(MouseButton::Left) {
            (-0.01, registry.get_block_id("air"), true)
        } else if mouse_buttons.just_pressed(MouseButton::Right) {
            let block = player_hotbar.slots[player_hotbar.active_slot];
            if block.is_none() {
                return;
            }
            (0.99, block.unwrap(), false)
        } else {
            return;
        };

    let ray = Ray3d::new(player_transform.translation, player_transform.forward());
    let filter = |id| query_chunk.contains(id);
    let raycast_setings = RayCastSettings::default().with_filter(&filter);

    if let Some((_, hit)) = ray_cast.cast_ray(ray, &raycast_setings).first() {
        let world_pos = (hit.point + hit.normal.normalize_or_zero() * normal_multiplier).floor();

        let cx = (world_pos[0] / storage.chunk_len as f32).floor() as i32;
        let cy = (world_pos[1] / storage.chunk_len as f32).floor() as i32;
        let cz = (world_pos[2] / storage.chunk_len as f32).floor() as i32;
        let local_x = (world_pos[0] as i32 - cx * storage.chunk_len as i32) as usize;
        let local_y = (world_pos[1] as i32 - cy * storage.chunk_len as i32) as usize;
        let local_z = (world_pos[2] as i32 - cz * storage.chunk_len as i32) as usize;

        if destroying
            || storage
                .get_voxel(&[cx, cy, cz], local_x, local_y, local_z)
                .is_some_and(|b| b == registry.get_block_id("air"))
        {
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
    }
}
