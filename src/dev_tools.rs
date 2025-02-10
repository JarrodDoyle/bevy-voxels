//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::{
        states::log_transitions,
        ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    },
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::common_conditions::input_just_pressed,
    pbr::wireframe::WireframeConfig,
    prelude::*,
    window::PresentMode,
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((DebugUiPlugin, FrameTimeDiagnosticsPlugin));
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(
        Update,
        (
            toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
            toggle_vsync.run_if(input_just_pressed(KeyCode::KeyV)),
            toggle_wireframe.run_if(input_just_pressed(KeyCode::KeyM)),
        ),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn toggle_vsync(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.present_mode = match window.present_mode {
        PresentMode::AutoVsync => PresentMode::AutoNoVsync,
        _ => PresentMode::AutoVsync,
    };
    info!("PRESENT_MODE: {:?}", window.present_mode);
}

fn toggle_wireframe(mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = !wireframe_config.global;
}
