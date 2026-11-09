use super::{
    EvidenceReportDeclaration, EvidenceReportErrorKind, EvidenceReportFieldKind,
    EvidenceReportScope,
};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

#[test]
fn query_owned_support_matrix_shape_is_declared_once_and_sealed() {
    let support_contract = evidence_identity("support-contract");
    let report = EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-query.runtime-support-matrix").unwrap(),
        "RuntimePublicSupportMatrixRow",
    )
    .unwrap()
    .shape_participating("surface", "branch-preview")
    .unwrap()
    .shape_participating("status", "supported")
    .unwrap()
    .bool_participating("parallel_api_forbidden", true)
    .unwrap()
    .bool_participating("admission_fail_closed", false)
    .unwrap()
    .identity_participating("support_contract", &support_contract)
    .unwrap()
    .diagnostic_value_nonparticipating("operator_note", "visible in inventory only")
    .unwrap()
    .seal()
    .unwrap();

    assert_eq!(report.report_name(), "RuntimePublicSupportMatrixRow");
    assert_eq!(report.indexed_field_count(), report.fields().len());
    assert_eq!(
        report.field("surface").unwrap().kind(),
        EvidenceReportFieldKind::Shape
    );
    assert_eq!(
        report.field("surface").unwrap().as_shape(),
        Some("branch-preview")
    );
    assert_eq!(
        report.field("parallel_api_forbidden").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        report.report_identity().scope(),
        WorthQueryEvidenceScope::ConsumerEvidenceReport
    );
    assert_eq!(
        report.field_inventory_identity().scope(),
        WorthQueryEvidenceScope::ConsumerEvidenceReportFieldInventory
    );
    assert_eq!(
        report.digest_participation_identity().scope(),
        WorthQueryEvidenceScope::ConsumerEvidenceReportDigestParticipation
    );
}

#[test]
fn branch_preview_basis_shape_covers_admitted_and_rejected_variants() {
    let preview_admission = evidence_identity("preview-admission");
    let branch_admission = evidence_identity("branch-admission");
    let admitted = branch_preview_report_shape(
        &preview_admission,
        &branch_admission,
        Some("direct_world"),
        ["direct_world"],
        None::<&str>,
    );
    let rejected = branch_preview_report_shape(
        &preview_admission,
        &branch_admission,
        None::<&str>,
        ["direct_world", "exact_support"],
        Some("world_space_exhausted"),
    );

    assert_ne!(admitted.report_identity(), rejected.report_identity());
    assert_eq!(
        admitted.field_inventory_identity(),
        rejected.field_inventory_identity()
    );
    assert_eq!(
        admitted.digest_participation_identity(),
        rejected.digest_participation_identity()
    );
    assert_eq!(
        rejected.field("exhaustion_reason").unwrap().kind(),
        EvidenceReportFieldKind::OptionalValue
    );
}

#[test]
fn participating_value_changes_report_identity_not_inventory_identity() {
    let left = minimal_report("supported", "diagnostic-a");
    let right = minimal_report("deferred", "diagnostic-a");

    assert_ne!(left.report_identity(), right.report_identity());
    assert_eq!(
        left.field_inventory_identity(),
        right.field_inventory_identity()
    );
    assert_eq!(
        left.digest_participation_identity(),
        right.digest_participation_identity()
    );
}

#[test]
fn diagnostic_nonparticipating_value_does_not_change_report_identity() {
    let left = minimal_report("supported", "diagnostic-a");
    let right = minimal_report("supported", "diagnostic-b");

    assert_eq!(left.report_identity(), right.report_identity());
    assert_ne!(
        left.field_inventory_identity(),
        right.field_inventory_identity()
    );
    assert_eq!(
        left.digest_participation_identity(),
        right.digest_participation_identity()
    );
}

#[test]
fn declaration_rejects_duplicate_field_names_before_sealing() {
    let error = EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-query.duplicate-proof").unwrap(),
        "DuplicateProof",
    )
    .unwrap()
    .shape_participating("surface", "read")
    .unwrap()
    .value_participating("surface", "write")
    .expect_err("duplicate field should reject");

    assert_eq!(error.kind(), EvidenceReportErrorKind::DuplicateFieldName);
}

#[test]
fn report_without_participating_fields_is_rejected() {
    let error = EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-query.empty-participation").unwrap(),
        "DiagnosticOnlyReport",
    )
    .unwrap()
    .diagnostic_value_nonparticipating("note", "not identity")
    .unwrap()
    .seal()
    .expect_err("diagnostic-only report cannot seal");

    assert_eq!(
        error.kind(),
        EvidenceReportErrorKind::MissingParticipatingField
    );
}

#[test]
fn field_order_changes_inventory_and_report_identity() {
    let left = EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-query.order-proof").unwrap(),
        "OrderProof",
    )
    .unwrap()
    .shape_participating("first", "a")
    .unwrap()
    .shape_participating("second", "b")
    .unwrap()
    .seal()
    .unwrap();
    let right = EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-query.order-proof").unwrap(),
        "OrderProof",
    )
    .unwrap()
    .shape_participating("second", "b")
    .unwrap()
    .shape_participating("first", "a")
    .unwrap()
    .seal()
    .unwrap();

    assert_ne!(
        left.field_inventory_identity(),
        right.field_inventory_identity()
    );
    assert_ne!(left.report_identity(), right.report_identity());
}

fn minimal_report(status: &str, diagnostic_note: &str) -> super::EvidenceReport {
    EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-query.minimal-report").unwrap(),
        "MinimalReport",
    )
    .unwrap()
    .shape_participating("status", status)
    .unwrap()
    .diagnostic_value_nonparticipating("note", diagnostic_note)
    .unwrap()
    .seal()
    .unwrap()
}

fn branch_preview_report_shape<'a, I>(
    preview_admission: &WorthQueryEvidenceIdentity,
    branch_admission: &WorthQueryEvidenceIdentity,
    realization_strategy: Option<&str>,
    attempted_strategies: I,
    exhaustion_reason: Option<&str>,
) -> super::EvidenceReport
where
    I: IntoIterator<Item = &'a str>,
{
    EvidenceReportDeclaration::new(
        EvidenceReportScope::new("worth-kernel.branch-preview-basis").unwrap(),
        "BranchPreviewBasisReport",
    )
    .unwrap()
    .shape_participating("family", "regular_pyramid")
    .unwrap()
    .identity_participating("preview_admission", preview_admission)
    .unwrap()
    .identity_participating("branch_admission", branch_admission)
    .unwrap()
    .optional_value_participating("realization_strategy", realization_strategy)
    .unwrap()
    .value_sequence_participating("attempted_realization_strategies", attempted_strategies)
    .unwrap()
    .optional_value_participating("exhaustion_reason", exhaustion_reason)
    .unwrap()
    .bool_participating("parity_verified", true)
    .unwrap()
    .seal()
    .unwrap()
}

fn evidence_identity(value: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimePublicSupportMatrixRow)
        .field_value(WorthQueryEvidenceTag::new("value"), value)
        .seal()
        .expect("fixture digest must fit")
}
