use std::time::Duration;

use bevy::{diagnostic::DiagnosticsStore, prelude::*, time::common_conditions::on_timer};

use crate::screens::Screen;

pub struct DebugStatsUiPlugin;

impl Plugin for DebugStatsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            update_ui.run_if(on_timer(Duration::from_secs_f32(0.1))),
        );
    }
}

#[derive(Component)]
pub struct DebugStatsUi;

fn setup(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Gameplay),
        DebugStatsUi,
        Node {
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ));
}

fn update_ui(
    mut commands: Commands,
    diagnostics: Res<DiagnosticsStore>,
    query_ui: Query<Entity, With<DebugStatsUi>>,
) {
    let debug_ui_id = query_ui.single();

    commands
        .entity(debug_ui_id)
        .despawn_descendants()
        .with_children(|parent| {
            for diagnostic in diagnostics.iter() {
                if !diagnostic.is_enabled {
                    continue;
                }

                let Some(current) = diagnostic.smoothed() else {
                    continue;
                };

                let name = diagnostic.path();
                if let Some(average) = diagnostic.average() {
                    parent.spawn(Text::new(format!(
                        "{name}: {current:.0} (avg: {average:.0})"
                    )));
                } else {
                    parent.spawn(Text::new(format!("{name}: {current:.0}")));
                }
            }
        });
}
