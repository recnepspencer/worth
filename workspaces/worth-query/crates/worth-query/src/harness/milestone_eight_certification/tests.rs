use super::{MilestoneEightCertificationAdapter, MilestoneEightFailureClass};
use crate::harness::certification::{
    contains_row, milestone_eight_requirements, unmet_required_rows,
};

#[test]
fn milestone_eight_certification_adapter_emits_named_matrix() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = MilestoneEightCertificationAdapter::
        scope_template_view_shape_semantic_parity_certification_artifact(resource_request);

    assert_eq!(
        artifact.suite_name,
        "Scope / Template / View-Shape Semantic Parity Test"
    );
    assert!(!artifact.certification_bundle_digest.is_empty());
    assert!(!artifact.coverage_matrix_digest.is_empty());
}

#[test]
fn milestone_eight_certification_matrix_meets_required_rows() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );
    let requirements = milestone_eight_requirements();
    let missing = unmet_required_rows(
        &matrix,
        requirements.required_canonical_rows,
        requirements.required_rejection_rows,
    );

    assert!(
        missing.is_empty(),
        "missing milestone eight certification rows: {missing:?}"
    );
}

#[test]
fn milestone_eight_certification_rows_have_required_outputs() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );

    for row in &matrix.rows {
        assert!(
            row.control_lane.has_required_outputs(),
            "control lane '{}' should have required outputs",
            row.row_name
        );
        assert!(
            row.hostile_lane.has_required_outputs(),
            "hostile lane '{}' should have required outputs",
            row.row_name
        );
        assert!(
            row.parity_lane.has_required_outputs(),
            "parity lane '{}' should have required outputs",
            row.row_name
        );
    }
}

#[test]
fn milestone_eight_certification_covers_named_semantic_rows() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );

    assert!(contains_row(&matrix, "direct-vs-scope-parity"));
    assert!(contains_row(&matrix, "direct-vs-template-parity"));
    assert!(contains_row(&matrix, "scope-template-direct-parity"));
    assert!(contains_row(
        &matrix,
        "kanban-desired-state-to-delta-parity"
    ));
    assert!(contains_row(&matrix, "kanban-delta-admission-boundary"));
    assert!(contains_row(&matrix, "grouped-delta-honesty"));
    assert!(contains_row(&matrix, "grouped-bridge-truth-view-authority"));
    assert!(contains_row(
        &matrix,
        "grouped-query-execution-surface-authority"
    ));
    assert!(contains_row(
        &matrix,
        "grouped-proof-chain-no-payload-rediscovery"
    ));
    assert!(contains_row(
        &matrix,
        "identity-aware-focused-inspector-parity"
    ));
    assert!(contains_row(
        &matrix,
        "identity-break-inspector-explicitness"
    ));
    assert!(contains_row(&matrix, "support-profile-honesty"));
}

#[test]
fn milestone_eight_saved_query_support_profile_drift_is_typed_rejection() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );
    let row = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "saved-query-support-profile-drift")
        .expect("saved query support profile drift row should exist");

    assert_eq!(
        row.hostile_lane.failure_class,
        MilestoneEightFailureClass::SavedQuerySupportProfileDrift
    );
}

#[test]
fn milestone_eight_deferred_and_grouped_rows_are_typed_rejections() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );
    let durable_row = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "durable-saved-query-deferred-debt")
        .expect("durable saved query deferred row should exist");
    let grouped_row = matrix
        .rejection_rows
        .iter()
        .find(|row| row.row_name == "grouped-hidden-refresh-forbidden")
        .expect("grouped hidden refresh forbidden row should exist");

    assert_eq!(
        durable_row.hostile_lane.failure_class,
        MilestoneEightFailureClass::DurableSavedQueryDeferredDebt
    );
    assert_eq!(
        grouped_row.hostile_lane.failure_class,
        MilestoneEightFailureClass::GroupedHiddenRefreshForbidden
    );
}

#[test]
fn milestone_eight_grouped_delta_row_is_non_cosmetic() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-delta-honesty")
        .expect("grouped delta honesty row should exist");

    assert_ne!(
        row.control_lane.delivery_digest,
        row.hostile_lane.delivery_digest
    );
    assert_ne!(
        row.control_lane.counter_snapshot_digest,
        row.hostile_lane.counter_snapshot_digest
    );
}

#[test]
fn milestone_eight_grouped_proof_chain_rows_are_present_and_stable() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );

    let truth_view_row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-bridge-truth-view-authority")
        .expect("grouped bridge truth-view authority row should exist");
    let execution_row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-query-execution-surface-authority")
        .expect("grouped execution surface authority row should exist");
    let rediscovery_row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "grouped-proof-chain-no-payload-rediscovery")
        .expect("grouped proof-chain row should exist");

    assert_eq!(
        truth_view_row.control_lane.delivery_digest,
        truth_view_row.parity_lane.delivery_digest
    );
    assert_ne!(
        truth_view_row.control_lane.delivery_digest,
        truth_view_row.hostile_lane.delivery_digest
    );
    assert_eq!(
        execution_row.control_lane.delivery_digest,
        execution_row.parity_lane.delivery_digest
    );
    assert_ne!(
        execution_row.control_lane.delivery_digest,
        execution_row.hostile_lane.delivery_digest
    );
    assert_eq!(
        rediscovery_row.control_lane.counter_snapshot_digest,
        rediscovery_row.parity_lane.counter_snapshot_digest
    );
    assert_ne!(
        rediscovery_row.control_lane.counter_snapshot_digest,
        rediscovery_row.hostile_lane.counter_snapshot_digest
    );
    assert!(!rediscovery_row
        .control_lane
        .counter_snapshot_digest
        .is_empty());
}

#[test]
fn milestone_eight_parity_rows_are_actually_adversarial() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );

    let scope_row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "direct-vs-scope-parity")
        .expect("scope parity row should exist");
    let template_row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "direct-vs-template-parity")
        .expect("template parity row should exist");
    let boundary_row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "kanban-delta-admission-boundary")
        .expect("kanban boundary row should exist");

    assert_eq!(
        scope_row.control_lane.query_digest,
        scope_row.hostile_lane.query_digest
    );
    assert_eq!(
        template_row.control_lane.query_digest,
        template_row.hostile_lane.query_digest
    );
    assert_ne!(
        boundary_row.control_lane.delivery_digest,
        boundary_row.hostile_lane.delivery_digest
    );
    assert_ne!(
        boundary_row.control_lane.counter_snapshot_digest,
        boundary_row.hostile_lane.counter_snapshot_digest
    );
}

#[test]
fn milestone_eight_identity_aware_inspector_rows_preserve_identity_classification() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );

    let parity = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "identity-aware-focused-inspector-parity")
        .expect("identity-aware inspector parity row should exist");
    assert!(!parity.control_lane.identity_consumption_digest.is_empty());
    assert!(!parity.control_lane.inspector_identity_digest.is_empty());
    assert_eq!(
        parity.control_lane.inspector_identity_classification,
        "authoritative_continuity"
    );
    assert_ne!(
        parity.control_lane.identity_consumption_digest,
        parity.control_lane.inspector_identity_digest
    );
    assert_eq!(
        parity.hostile_lane.inspector_identity_classification,
        "advisory_candidates"
    );
    assert_ne!(
        parity.control_lane.inspector_identity_digest,
        parity.hostile_lane.inspector_identity_digest
    );
    assert_eq!(
        parity.control_lane.inspector_identity_digest,
        parity.parity_lane.inspector_identity_digest
    );
    assert_eq!(
        parity.control_lane.inspector_identity_classification,
        parity.parity_lane.inspector_identity_classification
    );

    let identity_break = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "identity-break-inspector-explicitness")
        .expect("identity-break inspector row should exist");
    assert_eq!(
        identity_break
            .hostile_lane
            .inspector_identity_classification,
        "identity_break"
    );
    assert_ne!(
        identity_break.hostile_lane.inspector_identity_digest,
        "none"
    );
    assert_ne!(
        identity_break.hostile_lane.identity_consumption_digest,
        identity_break.hostile_lane.inspector_identity_digest
    );
    assert_eq!(
        identity_break
            .hostile_lane
            .inspector_identity_classification,
        identity_break.parity_lane.inspector_identity_classification
    );
}

#[test]
fn milestone_eight_support_profile_row_tracks_full_report_state() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = MilestoneEightCertificationAdapter::scope_template_view_shape_semantic_parity_test(
        resource_request,
    );
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "support-profile-honesty")
        .expect("support profile honesty row should exist");

    assert_eq!(
        row.control_lane.query_digest,
        row.control_lane.support_profile_digest
    );
    assert_eq!(
        row.parity_lane.query_digest,
        row.parity_lane.support_profile_digest
    );
    assert_ne!(
        row.control_lane.support_profile_digest,
        row.hostile_lane.support_profile_digest
    );
    assert_ne!(
        row.control_lane.artifact_binding_matrix_digest,
        row.hostile_lane.artifact_binding_matrix_digest
    );
}
