use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, RegisterDiagnostic},
    prelude::*,
};

pub const MESHING_TIME_DIAGNOSTIC: DiagnosticPath = DiagnosticPath::const_new("Meshing time");

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(MESHING_TIME_DIAGNOSTIC).with_suffix("μs"));
    }
}
