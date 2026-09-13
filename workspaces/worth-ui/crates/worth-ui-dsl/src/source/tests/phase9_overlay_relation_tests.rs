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

#[test]
fn distinct_portal_backdrops_compile_without_inventing_static_stack_order() {
    let source = r#"
        surface pulse.surface {}
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        portal first { surface pulse.surface anchor first.target layer modal dismiss escape focus first_enabled motion system_popover }
        portal second { surface pulse.surface anchor second.target layer modal dismiss escape focus first_enabled motion system_popover }
        backdrop lower {
            scope per_portal_instance first
            extent surface_viewport pulse.surface
            presence while portal first presented
            motion none
            place immediately_before portal first
            appearance { role overlay.scrim }
        }
        backdrop upper {
            scope per_portal_instance second
            extent surface_viewport pulse.surface
            presence while portal second presented
            motion none
            place immediately_before portal second
            appearance { role overlay.scrim }
        }
    "#;
    compile_file(source).expect("live Portal order completes the two immediate groups");
    let ambiguous = source
        .replace(
            "place immediately_before portal first",
            "place above_surface_content",
        )
        .replace(
            "place immediately_before portal second",
            "place above_surface_content",
        );
    let report = compile_file(&ambiguous).expect_err("unanchored backdrops still have no order");
    assert_eq!(
        report.diagnostics()[0].identity().code(),
        WorthUiDslCompileDiagnosticCode::AmbiguousOverlayRelation
    );
}
