use std::path::PathBuf;

use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisPredicate,
    UiAppearanceDecisionPartitionDenial, WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnostic,
    WorthUiDslCompileDiagnosticCode, WorthUiDslCompileDiagnosticDetail, WorthUiDslCompileReport,
    WorthUiDslCompileStopClass, WorthUiDslCompiler,
};

fn compile(source: &str) -> WorthUiDslCompileReport {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
    .expect_err("Gate 2 denial fixture should be rejected")
}

fn only_diagnostic(source: &str) -> WorthUiDslCompileDiagnostic {
    let report = compile(source);
    assert_eq!(report.diagnostics().len(), 1);
    report.diagnostics()[0].clone()
}

fn assert_location(
    diagnostic: &WorthUiDslCompileDiagnostic,
    stop_class: WorthUiDslCompileStopClass,
) {
    assert_eq!(diagnostic.stop_class(), stop_class);
    assert_eq!(diagnostic.identity().module_id(), Some("app/main.wui"));
    assert!(diagnostic.identity().span().is_some());
}

#[test]
fn malformed_appearance_declaration_is_denied_at_the_compiler_boundary() {
    let diagnostic = only_diagnostic(
        r#"appearance role malformed applies_to platform.control.activation {
            background
        }"#,
    );
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::InvalidAppearanceDeclaration
    );
    assert_location(&diagnostic, WorthUiDslCompileStopClass::LanguageLegality);
}

#[test]
fn ambiguous_partition_is_denied_with_the_partition_variant() {
    let diagnostic = only_diagnostic(
        r#"appearance role ambiguous applies_to platform.control.activation {
            background over [hover] {
                cell outside when hover = outside use token(control.background)
                otherwise use token(control.background)
                otherwise use token(control.background.focused)
            }
        }"#,
    );
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::AmbiguousAppearanceDeclaration
    );
    assert_location(&diagnostic, WorthUiDslCompileStopClass::LanguageLegality);
    let Some(WorthUiDslCompileDiagnosticDetail::AppearancePartition {
        role,
        aspect,
        expected_kind,
        denial,
        source_span,
    }) = diagnostic.detail()
    else {
        panic!("ambiguous partition must retain typed partition facts");
    };
    assert_eq!(role.as_str(), "ambiguous");
    assert_eq!(*aspect, UiAppearanceAspect::Background);
    assert_eq!(*expected_kind, crate::UiThemeValueKind::Color);
    assert_eq!(
        *denial,
        UiAppearanceDecisionPartitionDenial::DuplicateOtherwise
    );
    assert!(source_span.is_some());
}

#[test]
fn overlapping_partition_is_denied_instead_of_using_source_order() {
    let diagnostic = only_diagnostic(
        r#"appearance role overlapping applies_to platform.control.activation {
            background over [hover] {
                cell first when hover = outside use token(control.background)
                cell second when hover = outside use token(control.background.focused)
                otherwise same_as first
            }
        }"#,
    );
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::OverlappingAppearanceCells
    );
    assert_location(&diagnostic, WorthUiDslCompileStopClass::LanguageLegality);
    let Some(WorthUiDslCompileDiagnosticDetail::AppearancePartition { denial, .. }) =
        diagnostic.detail()
    else {
        panic!("overlapping partition must retain typed partition facts");
    };
    assert_eq!(
        *denial,
        UiAppearanceDecisionPartitionDenial::OverlappingCell
    );
}

#[test]
fn incomplete_coverage_reports_the_uncovered_cell_and_exact_repair() {
    let source = r#"appearance role incomplete applies_to platform.control.activation {
        background over [operability, focus, validation] {
            cell base when operability = ready, focus = unfocused, validation = unspecified
                use token(control.background)
        }
    }"#;
    let diagnostic = only_diagnostic(source);
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::MissingAppearanceCoverage
    );
    assert_location(&diagnostic, WorthUiDslCompileStopClass::LanguageLegality);
    let Some(WorthUiDslCompileDiagnosticDetail::MissingAppearanceCoverage {
        role,
        aspect,
        expected_kind,
        uncovered_canonical_state_cell,
        exact_repair_predicate,
        source_span,
    }) = diagnostic.detail()
    else {
        panic!("missing coverage must retain its exact repair facts");
    };
    assert_eq!(role.as_str(), "incomplete");
    assert_eq!(*aspect, UiAppearanceAspect::Background);
    assert_eq!(*expected_kind, crate::UiThemeValueKind::Color);
    assert_eq!(
        uncovered_canonical_state_cell.classes(),
        &[
            UiAppearanceAxisClass::OperabilityReady,
            UiAppearanceAxisClass::FocusUnfocused,
            UiAppearanceAxisClass::ValidationValid,
        ]
    );
    assert_eq!(
        exact_repair_predicate.predicates(),
        &[
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::OperabilityReady),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::FocusUnfocused),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::ValidationValid),
        ]
    );
    assert_eq!(
        source_span
            .as_ref()
            .expect("coverage detail should retain the aspect span")
            .start_byte(),
        source
            .find("background")
            .expect("background should be present")
    );
}

#[test]
fn wrong_value_kind_is_denied_as_a_typed_appearance_failure() {
    let diagnostic = only_diagnostic(
        r#"appearance role wrong-kind applies_to platform.control.activation {
            background use transparent-opacity
        }"#,
    );
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind
    );
    assert_location(&diagnostic, WorthUiDslCompileStopClass::LanguageLegality);
}

#[test]
fn saturated_partition_is_denied_before_canonical_cells_are_materialized() {
    let diagnostic = only_diagnostic(
        r#"appearance role saturated applies_to platform.control.activation {
            background over [operability, focus, validation, selection] {
                otherwise use token(control.background)
            }
        }"#,
    );
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied
    );
    assert_location(&diagnostic, WorthUiDslCompileStopClass::LanguageLegality);
    let Some(WorthUiDslCompileDiagnosticDetail::AppearancePartition {
        role,
        aspect,
        expected_kind,
        denial,
        source_span,
    }) = diagnostic.detail()
    else {
        panic!("capacity denial must retain typed partition facts");
    };
    assert_eq!(role.as_str(), "saturated");
    assert_eq!(*aspect, UiAppearanceAspect::Background);
    assert_eq!(*expected_kind, crate::UiThemeValueKind::Color);
    assert_eq!(
        *denial,
        UiAppearanceDecisionPartitionDenial::CellCapacityExceeded
    );
    assert!(source_span.is_some());
}

fn overlay_source(backdrops: &str) -> String {
    format!(
        r#"
        surface pulse.surface {{}}
        appearance role overlay.scrim applies_to backdrop {{
            background use token(overlay.background)
            opacity use token(overlay.opacity)
        }}
        {backdrops}
        "#
    )
}

fn backdrop_declaration(placement: &str, identity: &str) -> String {
    format!(
        r#"backdrop {identity} {{
            scope surface_singleton
            extent surface_viewport pulse.surface
            presence always
            motion none
            place {placement}
            appearance {{ role overlay.scrim }}
        }}"#
    )
}

#[test]
fn ambiguous_overlay_is_denied_with_a_typed_overlay_variant() {
    let source = overlay_source(&format!(
        "{}\n{}",
        backdrop_declaration("above_surface_content", "first"),
        backdrop_declaration("above_surface_content", "second")
    ));
    let diagnostic = only_diagnostic(&source);
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::AmbiguousOverlayRelation
    );
    assert_location(
        &diagnostic,
        WorthUiDslCompileStopClass::SemanticNormalization,
    );
}

#[test]
fn missing_overlay_anchor_is_denied_before_relation_graph_sealing() {
    let source = overlay_source(&backdrop_declaration(
        "immediately_before backdrop missing",
        "first",
    ));
    let diagnostic = only_diagnostic(&source);
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::MissingOverlayAnchor
    );
    assert_location(
        &diagnostic,
        WorthUiDslCompileStopClass::SemanticNormalization,
    );
}

#[test]
fn cyclic_overlay_relation_is_denied_before_relation_graph_sealing() {
    let source = r#"
    surface pulse.surface {}
    appearance role overlay.scrim applies_to backdrop {
        background use token(overlay.background)
        opacity use token(overlay.opacity)
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
    "#;
    let diagnostic = only_diagnostic(source);
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::CyclicOverlayRelation
    );
    assert_location(
        &diagnostic,
        WorthUiDslCompileStopClass::SemanticNormalization,
    );
}
