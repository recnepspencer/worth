use std::path::PathBuf;

use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearanceCellReferenceOrigin, UiAppearanceCellReferenceRepair,
    UiAppearanceDecisionPartitionDenial, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleIdentity, UiAppearanceStateAxis, UiThemeSlotIdentity, UiThemeValueKind,
    WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileDiagnosticDetail,
    WorthUiDslCompiler,
};

fn compile(source: &str) -> crate::WorthUiDslCompileReport {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
    .expect_err("diagnostic fixture should be rejected")
}

fn slot(name: &str) -> UiThemeSlotIdentity {
    UiThemeSlotIdentity::new(name).unwrap()
}

#[test]
fn missing_coverage_preserves_role_aspect_kind_cell_repair_and_aspect_span() {
    let source = r#"appearance role incomplete applies_to button {
        background over [hover] {
            cell outside when hover = outside use token(button.background)
        }
    }"#;
    let report = compile(source);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::MissingAppearanceCoverage
    );
    let Some(WorthUiDslCompileDiagnosticDetail::MissingAppearanceCoverage {
        role,
        aspect,
        expected_kind,
        uncovered_canonical_state_cell,
        exact_repair_predicate,
        source_span,
    }) = diagnostic.detail()
    else {
        panic!("missing coverage must carry its typed admission facts");
    };
    assert_eq!(role.as_str(), "incomplete");
    assert_eq!(*aspect, UiAppearanceAspect::Background);
    assert_eq!(*expected_kind, UiThemeValueKind::Color);
    assert_eq!(
        uncovered_canonical_state_cell.classes(),
        &[UiAppearanceAxisClass::Hovered]
    );
    assert_eq!(
        exact_repair_predicate.predicates(),
        &[UiAppearanceAxisPredicate::exact(
            UiAppearanceAxisClass::Hovered
        )]
    );
    let span = source_span.as_ref().expect("aspect token span is required");
    assert_eq!(span.module_id(), "app/main.wui");
    assert_eq!(
        span.start_byte(),
        source.find("background").expect("background token exists")
    );
}

#[test]
fn missing_same_as_preserves_name_origin_and_reference_token_span() {
    let source = r#"appearance role missing-ref applies_to button {
        background over [hover] {
            cell outside when hover = outside use token(button.background)
            otherwise same_as missing
        }
    }"#;
    let report = compile(source);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::MissingAppearanceCellReference
    );
    let Some(WorthUiDslCompileDiagnosticDetail::MissingAppearanceCellReference {
        role,
        aspect,
        expected_kind,
        referenced_cell_name,
        reference_origin,
        repair,
        source_span,
    }) = diagnostic.detail()
    else {
        panic!("missing same_as must carry its typed reference facts");
    };
    assert_eq!(role.as_str(), "missing-ref");
    assert_eq!(*aspect, UiAppearanceAspect::Background);
    assert_eq!(*expected_kind, UiThemeValueKind::Color);
    assert_eq!(referenced_cell_name.as_ref(), "missing");
    assert_eq!(
        *reference_origin,
        UiAppearanceCellReferenceOrigin::OtherwiseClause
    );
    assert_eq!(
        *repair,
        UiAppearanceCellReferenceRepair::DeclareNamedCellOrRetargetReference
    );
    assert!(diagnostic
        .message()
        .contains(&format!("lawful repair: {}", repair.render())));
    let span = source_span.as_ref().expect("same_as name span is required");
    assert_eq!(
        span.start_byte(),
        source.rfind("missing").expect("reference token exists")
    );
}

#[test]
fn missing_named_cell_reference_carries_the_same_lawful_repair() {
    let source = r#"appearance role missing-named applies_to button {
        background over [hover] {
            cell outside when hover = outside use token(button.background)
            cell inside when hover = hovered use same_as missing
            otherwise use token(button.background)
        }
    }"#;
    let report = compile(source);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(
        diagnostic.identity().code(),
        WorthUiDslCompileDiagnosticCode::MissingAppearanceCellReference
    );
    let Some(WorthUiDslCompileDiagnosticDetail::MissingAppearanceCellReference {
        role,
        aspect,
        expected_kind,
        referenced_cell_name,
        reference_origin,
        repair,
        source_span,
    }) = diagnostic.detail()
    else {
        panic!("named-cell same_as must carry its typed repair facts");
    };
    assert_eq!(role.as_str(), "missing-named");
    assert_eq!(*aspect, UiAppearanceAspect::Background);
    assert_eq!(*expected_kind, UiThemeValueKind::Color);
    assert_eq!(referenced_cell_name.as_ref(), "missing");
    assert_eq!(
        *reference_origin,
        UiAppearanceCellReferenceOrigin::NamedCell
    );
    assert_eq!(
        *repair,
        UiAppearanceCellReferenceRepair::DeclareNamedCellOrRetargetReference
    );
    assert!(diagnostic
        .message()
        .contains(&format!("lawful repair: {}", repair.render())));
    let span = source_span.as_ref().expect("same_as name span is required");
    assert_eq!(
        span.start_byte(),
        source.rfind("missing").expect("reference token exists")
    );
}

#[test]
fn one_hole_repair_predicate_covers_only_the_uncovered_cell() {
    let domain = UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover);
    let value = crate::UiAppearanceDecisionResult::theme_slot(
        slot("button.background"),
        UiThemeValueKind::Color,
    );
    let denial = UiAppearancePartitionAuthoring::new([domain])
        .with_cell(UiAppearanceCell::new(
            None::<&str>,
            [UiAppearanceAxisPredicate::exact(
                UiAppearanceAxisClass::HoverOutside,
            )],
            crate::UiAppearanceCellValue::ThemeSlot {
                slot: slot("button.outside"),
                value_kind: UiThemeValueKind::Color,
            },
        ))
        .compile(UiAppearanceAspect::Background)
        .expect_err("one hover cell must leave one hole");
    let UiAppearanceDecisionPartitionDenial::MissingCell {
        uncovered_canonical_state_cell,
        exact_repair_predicate,
    } = denial
    else {
        panic!("fixture must produce a missing-cell denial");
    };
    assert_eq!(
        uncovered_canonical_state_cell.classes(),
        &[UiAppearanceAxisClass::Hovered]
    );
    let repaired = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Hover,
    )])
    .with_cell(UiAppearanceCell::new(
        None::<&str>,
        [UiAppearanceAxisPredicate::exact(
            UiAppearanceAxisClass::HoverOutside,
        )],
        crate::UiAppearanceCellValue::theme_slot(slot("button.outside"), UiThemeValueKind::Color),
    ))
    .with_cell(UiAppearanceCell::new(
        None::<&str>,
        exact_repair_predicate.predicates().iter().copied(),
        crate::UiAppearanceCellValue::theme_slot(value.slot().unwrap().clone(), value.value_kind()),
    ))
    .compile(UiAppearanceAspect::Background)
    .expect("the exact repair predicate should close exactly one hole");
    assert_eq!(repaired.cells().len(), 2);
}

#[test]
fn file_missing_partition_facts_match_rust_authoring_facts_without_a_source_span() {
    let rust_denial =
        UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("rust-missing").unwrap())
            .cover(
                UiAppearanceAspect::Background,
                UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
                    UiAppearanceStateAxis::Hover,
                )])
                .with_cell(UiAppearanceCell::new(
                    None::<&str>,
                    [UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::HoverOutside,
                    )],
                    crate::UiAppearanceCellValue::theme_slot(
                        slot("rust.background"),
                        UiThemeValueKind::Color,
                    ),
                )),
            )
            .expect_err("Rust authoring must expose the same missing coverage denial");
    let crate::UiAppearanceRoleAuthoringDenial::Partition(denial) = rust_denial else {
        panic!("expected a typed partition admission denial");
    };
    assert_eq!(denial.role().as_str(), "rust-missing");
    assert_eq!(denial.aspect(), UiAppearanceAspect::Background);
    assert_eq!(denial.expected_kind(), UiThemeValueKind::Color);
    assert!(matches!(
        denial.denial(),
        UiAppearanceDecisionPartitionDenial::MissingCell { .. }
    ));
}
