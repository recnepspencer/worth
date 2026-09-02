use std::path::PathBuf;

use crate::{WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnosticCode, WorthUiDslCompiler};

fn compile_file(
    source: &str,
) -> Result<crate::WorthUiSealedSemanticPackage, crate::WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
}

#[test]
fn overlay_cycles_are_rejected_before_any_runtime_plan_exists() {
    let report = compile_file(
        r#"
        surface pulse.surface {}
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop first {
            scope surface_singleton
            extent surface_viewport pulse.surface
            presence always
            motion none
            place immediately_before backdrop second
            appearance { role overlay.scrim }
        }
        backdrop second {
            scope surface_singleton
            extent surface_viewport pulse.surface
            presence always
            motion none
            place immediately_before backdrop first
            appearance { role overlay.scrim }
        }
        "#,
    )
    .expect_err("overlay cycle must be denied");
    assert_eq!(
        report.diagnostics()[0].identity().code(),
        WorthUiDslCompileDiagnosticCode::CyclicOverlayRelation
    );
}
