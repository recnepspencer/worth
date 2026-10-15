use super::MilestoneNineThreeCertificationAdapter;
use crate::harness::certification::{
    milestone_nine_three_requirements, unmet_required_assertion_classes, HostileExpectation,
    ParityAnchor, RequiredAssertionClass,
};

#[test]
fn milestone_nine_three_rows_enforce_required_assertion_classes() {
    let matrix = MilestoneNineThreeCertificationAdapter::
        query_subscription_bridge_parity_and_diagnostic_sufficiency_test();
    let mut covered = Vec::new();

    for row in &matrix.rows {
        let control = row.control_lane.semantic_signature();
        let hostile = row.hostile_lane.semantic_signature();
        let parity = row.parity_lane.semantic_signature();
        match row.hostile_expectation {
            HostileExpectation::EquivalentToControl => {
                assert_eq!(control, hostile, "row '{}'", row.row_name);
                covered.push(RequiredAssertionClass::Equality);
            }
            HostileExpectation::DistinctFromControl => {
                assert_ne!(control, hostile, "row '{}'", row.row_name);
                covered.push(RequiredAssertionClass::Inequality);
            }
        }
        match row.parity_anchor {
            ParityAnchor::Control => assert_eq!(parity, control, "row '{}'", row.row_name),
            ParityAnchor::Hostile => assert_eq!(parity, hostile, "row '{}'", row.row_name),
        }
    }

    for row in &matrix.rejection_rows {
        covered.push(RequiredAssertionClass::TypedFailure);
        if row.hostile_lane.counter_snapshot != row.control_lane.counter_snapshot {
            covered.push(RequiredAssertionClass::ZeroResidue);
        }
    }

    covered.sort();
    covered.dedup();
    let missing = unmet_required_assertion_classes(
        &covered,
        milestone_nine_three_requirements().required_assertion_classes,
    );
    assert!(missing.is_empty(), "missing assertion classes: {missing:?}");
}

#[test]
fn milestone_nine_three_grouped_and_inspector_rows_preserve_query_family_distinction() {
    let matrix = MilestoneNineThreeCertificationAdapter::
        query_subscription_bridge_parity_and_diagnostic_sufficiency_test();

    let detail = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "detail-family-support-and-parity")
        .unwrap();
    let ordered = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "ordered-collection-family-support-and-parity")
        .unwrap();
    let grouped = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-collection-family-support-and-parity")
        .unwrap();
    let inspector = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "inspector-family-support-and-parity")
        .unwrap();

    assert_ne!(
        ordered.control_lane.query_family_label,
        grouped.hostile_lane.query_family_label
    );
    assert_ne!(
        detail.control_lane.query_family_label,
        inspector.hostile_lane.query_family_label
    );
    assert_eq!(
        ordered.control_lane.bridge_family_label,
        grouped.hostile_lane.bridge_family_label
    );
    assert_eq!(
        detail.control_lane.bridge_family_label,
        inspector.hostile_lane.bridge_family_label
    );
    assert_ne!(
        ordered.control_lane.subscription_family_digest,
        grouped.hostile_lane.subscription_family_digest
    );
    assert_ne!(
        detail.control_lane.subscription_family_digest,
        inspector.hostile_lane.subscription_family_digest
    );
}

#[test]
fn milestone_nine_three_support_scale_row_keeps_query_meaning_fixed_while_cost_posture_changes() {
    let matrix = MilestoneNineThreeCertificationAdapter::
        query_subscription_bridge_parity_and_diagnostic_sufficiency_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "support-matrix-scale-honesty")
        .unwrap();

    assert_eq!(row.control_lane.query_digest, row.hostile_lane.query_digest);
    assert_eq!(
        row.control_lane.subscription_family_digest,
        row.hostile_lane.subscription_family_digest
    );
    assert_eq!(
        row.control_lane.subscription_declaration_digest,
        row.hostile_lane.subscription_declaration_digest
    );
    assert_eq!(
        row.control_lane.bridge_declaration_digest,
        row.hostile_lane.bridge_declaration_digest
    );
    assert_eq!(
        row.control_lane.signal_strategy_digest,
        row.hostile_lane.signal_strategy_digest
    );
    assert_eq!(
        row.control_lane.support_report_digest,
        row.hostile_lane.support_report_digest
    );
    assert_eq!(
        row.control_lane.support_matrix_digest,
        row.hostile_lane.support_matrix_digest
    );
    assert_eq!(
        row.control_lane.support_lookup_receipt_digest,
        row.hostile_lane.support_lookup_receipt_digest
    );
    assert_eq!(
        row.control_lane.manual_bridge_witness_digest,
        row.hostile_lane.manual_bridge_witness_digest
    );
    assert_eq!(
        row.control_lane.bridge_parity_digest,
        row.hostile_lane.bridge_parity_digest
    );
    assert_eq!(
        row.control_lane.diagnostic_trace_digest,
        row.hostile_lane.diagnostic_trace_digest
    );
    assert_eq!(
        row.control_lane.admitted_diagnostic_bundle_digest,
        row.hostile_lane.admitted_diagnostic_bundle_digest
    );
    assert_eq!(
        row.control_lane.lifecycle_certification_digest,
        row.hostile_lane.lifecycle_certification_digest
    );
    assert_ne!(
        row.control_lane.certification_coverage_receipt_digest,
        row.hostile_lane.certification_coverage_receipt_digest
    );
    assert_ne!(
        row.control_lane.coverage_resolution_posture_label,
        row.hostile_lane.coverage_resolution_posture_label
    );
    assert_ne!(
        row.control_lane.runtime_certification_bundle_digest,
        row.hostile_lane.runtime_certification_bundle_digest
    );
}

#[test]
fn milestone_nine_three_declaration_family_drift_row_distinguishes_runtime_churn_from_declaration_identity(
) {
    let matrix = MilestoneNineThreeCertificationAdapter::
        query_subscription_bridge_parity_and_diagnostic_sufficiency_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "declaration-family-drift-vs-lifecycle-churn-distinctness")
        .unwrap();

    assert_eq!(
        row.control_lane.subscription_declaration_digest,
        row.hostile_lane.subscription_declaration_digest
    );
    assert_ne!(
        row.control_lane.lifecycle_certification_digest,
        row.hostile_lane.lifecycle_certification_digest
    );
    assert_ne!(
        row.control_lane.runtime_certification_bundle_digest,
        row.hostile_lane.runtime_certification_bundle_digest
    );
}
