use super::super::{MilestoneNineCertificationAdapter, MilestoneNineFailureClass};
use super::semantic_signature;
use crate::harness::certification::{
    milestone_nine_requirements, unmet_required_assertion_classes, HostileExpectation,
    ParityAnchor, RequiredAssertionClass,
};

#[test]
fn milestone_nine_certification_rows_enforce_declared_lane_semantics() {
    let matrix = MilestoneNineCertificationAdapter::policy_tenant_context_admission_test();
    let mut covered = Vec::new();

    for row in &matrix.rows {
        let control = semantic_signature(&row.control_lane);
        let hostile = semantic_signature(&row.hostile_lane);
        let parity = semantic_signature(&row.parity_lane);
        match row.hostile_expectation {
            HostileExpectation::EquivalentToControl => {
                assert_eq!(
                    control, hostile,
                    "row '{}' declares hostile equivalence but emits different semantic evidence",
                    row.row_name
                );
                covered.push(RequiredAssertionClass::Equality);
            }
            HostileExpectation::DistinctFromControl => {
                assert_ne!(
                    control, hostile,
                    "row '{}' declares hostile distinction but emits identical semantic evidence",
                    row.row_name
                );
                covered.push(RequiredAssertionClass::Inequality);
            }
        }

        match row.parity_anchor {
            ParityAnchor::Control => assert_eq!(
                parity, control,
                "row '{}' parity lane must independently match the control anchor",
                row.row_name
            ),
            ParityAnchor::Hostile => assert_eq!(
                parity, hostile,
                "row '{}' parity lane must independently match the hostile anchor",
                row.row_name
            ),
        }
    }

    for row in &matrix.rejection_rows {
        assert!(
            row.control_lane.has_required_outputs(),
            "rejection row '{}' needs a successful control basis",
            row.row_name
        );
        assert!(
            row.parity_lane.has_required_outputs(),
            "rejection row '{}' needs a successful parity basis",
            row.row_name
        );
        assert!(
            !row.hostile_lane.failure_digest.is_empty()
                && !row.hostile_lane.counter_snapshot_digest.is_empty(),
            "rejection row '{}' must emit failure and counter evidence",
            row.row_name
        );
        assert_ne!(
            row.hostile_lane.counter_snapshot_digest, row.control_lane.counter_snapshot_digest,
            "rejection row '{}' must not reuse the control counter snapshot",
            row.row_name
        );
        covered.push(RequiredAssertionClass::TypedFailure);
    }

    let zero_residue = matrix.rejection_rows.iter().any(|row| {
        matches!(
            row.row_name,
            "relationship-proof-host-callback-forbidden"
                | "phase-two-no-truth-touch"
                | "phase-three-no-truth-touch-before-plan-admission"
        ) && row.hostile_lane.counter_snapshot_digest != row.control_lane.counter_snapshot_digest
    });
    if zero_residue {
        covered.push(RequiredAssertionClass::ZeroResidue);
    }
    covered.sort();
    covered.dedup();

    let missing = unmet_required_assertion_classes(
        &covered,
        milestone_nine_requirements().required_assertion_classes,
    );
    assert!(
        missing.is_empty(),
        "missing milestone nine assertion classes: {missing:?}"
    );
}

#[test]
fn milestone_nine_execution_seam_denials_are_typed() {
    let matrix = MilestoneNineCertificationAdapter::policy_tenant_context_admission_test();
    let live = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "live-subscription-deferred-before-truth")
        .expect("live seam denial row should exist");
    let diff = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "historical-diff-deferred-before-truth")
        .expect("diff seam denial row should exist");

    assert_eq!(
        live.hostile_lane.failure_class,
        MilestoneNineFailureClass::UnsupportedExecutionMode
    );
    assert_eq!(
        diff.hostile_lane.failure_class,
        MilestoneNineFailureClass::UnsupportedExecutionMode
    );
    assert_ne!(
        live.hostile_lane.counter_snapshot_digest,
        live.control_lane.counter_snapshot_digest
    );
}

#[test]
fn milestone_nine_policy_narrowing_changes_only_the_admission_disposition() {
    let matrix = MilestoneNineCertificationAdapter::policy_tenant_context_admission_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "policy-narrowing-disposition")
        .expect("policy narrowing row should exist");

    assert_eq!(
        row.control_lane.canonical_query_digest,
        row.hostile_lane.canonical_query_digest
    );
    assert_eq!(
        row.control_lane.tenant_truth_basis_digest,
        row.hostile_lane.tenant_truth_basis_digest
    );
    assert_ne!(
        row.control_lane.admission_disposition,
        row.hostile_lane.admission_disposition
    );
}

#[test]
fn milestone_nine_work_budget_is_part_of_admission_evidence() {
    let matrix = MilestoneNineCertificationAdapter::policy_tenant_context_admission_test();
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "policy-work-budget-explicitness")
        .expect("policy work budget row should exist");
    let unknown = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "unknown-policy-cost-denied-before-truth")
        .expect("unknown cost denial row should exist");

    assert_eq!(row.control_lane.policy_cost_posture, "constant_proof");
    assert!(!row.control_lane.policy_work_budget_digest.is_empty());
    assert_ne!(
        unknown.hostile_lane.counter_snapshot_digest,
        row.control_lane.counter_snapshot_digest
    );
}

#[test]
fn milestone_nine_runtime_store_and_durable_handoff_is_explicit() {
    let matrix = MilestoneNineCertificationAdapter::policy_tenant_context_admission_test();
    let handoff = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "policy-execution-handoff-honesty")
        .expect("handoff honesty row should exist");
    let durable_cursor = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "durable-policy-cursor-deferred")
        .expect("durable cursor deferred row should exist");
    let durable_reload = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "durable-policy-artifact-reload-deferred")
        .expect("durable artifact reload deferred row should exist");
    let durable_delivery = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "durable-policy-delivery-metadata-deferred")
        .expect("durable delivery metadata deferred row should exist");

    assert_ne!(
        handoff.control_lane.policy_execution_seam_digest,
        handoff.control_lane.support_profile_digest
    );
    assert_eq!(
        durable_cursor.hostile_lane.failure_class,
        MilestoneNineFailureClass::PolicyExecutionSeamDenied
    );
    assert_ne!(
        durable_cursor.hostile_lane.counter_snapshot_digest,
        durable_reload.hostile_lane.counter_snapshot_digest
    );
    assert_ne!(
        durable_reload.hostile_lane.counter_snapshot_digest,
        durable_delivery.hostile_lane.counter_snapshot_digest
    );
}
