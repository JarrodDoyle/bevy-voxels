//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::{
        states::log_transitions,
        ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    },
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_just_pressed,
    prelude::*,
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        DebugUiPlugin,
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin,
    ));
    app.add_systems(Update, log_transitions::<Screen>);
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
