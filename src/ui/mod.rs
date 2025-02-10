mod crosshair;
mod debug_stats;
mod hotbar;
mod loading_screen;
mod splash_screen;
mod theme;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            splash_screen::SplashScreenUiPlugin,
            loading_screen::LoadingScreenUiPlugin,
            crosshair::CrosshairUiPlugin,
            theme::plugin,
            hotbar::HotbarUiPlugin,
            debug_stats::DebugStatsUiPlugin,
        ));
    }
}
