use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, RegisterDiagnostic},
    prelude::*,
};

pub const MESHING_TIME_DIAGNOSTIC: DiagnosticPath = DiagnosticPath::const_new("Chunk Meshing");
pub const SAVE_TIME_DIAGNOSTIC: DiagnosticPath = DiagnosticPath::const_new("Chunk Saving");
pub const LOAD_TIME_DIAGNOSTIC: DiagnosticPath = DiagnosticPath::const_new("Chunk Loading");
pub const GEN_TIME_DIAGNOSTIC: DiagnosticPath = DiagnosticPath::const_new("Chunk Generation");

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(MESHING_TIME_DIAGNOSTIC).with_suffix("μs"));
        app.register_diagnostic(Diagnostic::new(SAVE_TIME_DIAGNOSTIC).with_suffix("μs"));
        app.register_diagnostic(Diagnostic::new(LOAD_TIME_DIAGNOSTIC).with_suffix("μs"));
        app.register_diagnostic(Diagnostic::new(GEN_TIME_DIAGNOSTIC).with_suffix("μs"));
    }
}
