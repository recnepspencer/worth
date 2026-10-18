use super::AspectApiFinalizationCertificationAdapter;
use crate::harness::certification::{
    unmet_required_assertion_classes, HostileExpectation, ParityAnchor, RequiredAssertionClass,
};

#[test]
fn aspect_api_finalization_rows_enforce_required_assertion_classes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );
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
        if row.hostile_lane.failure_digest != row.control_lane.receipt_digest {
            covered.push(RequiredAssertionClass::ZeroResidue);
        }
    }

    covered.sort();
    covered.dedup();
    let missing = unmet_required_assertion_classes(
        &covered,
        &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
    );
    assert!(missing.is_empty(), "missing assertion classes: {missing:?}");
}

#[test]
fn aspect_api_finalization_clear_row_proves_touched_scope_narrowing() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "typed-clear-narrows-by-touched-meaning")
        .expect("clear row should exist");

    assert_eq!(row.control_lane.routed_patch_count, 0);
    assert_eq!(row.hostile_lane.routed_patch_count, 1);
    assert_eq!(row.control_lane.affected_live_view_count, 0);
    assert_eq!(row.hostile_lane.affected_live_view_count, 1);
}

#[test]
fn aspect_api_finalization_preview_batch_row_proves_lane_isolation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = AspectApiFinalizationCertificationAdapter::public_aspect_api_finalization_test(
        request_execution,
    );
    let row = matrix
        .rows
        .iter()
        .find(|row| row.row_name == "preview-batch-lane-isolation")
        .expect("preview batch row should exist");

    assert_eq!(row.control_lane.authority_lane_label, "authoritative-truth");
    assert_eq!(row.hostile_lane.authority_lane_label, "preview-truth");
    assert_eq!(row.hostile_lane.preview_residue_count, 0);
    assert_eq!(row.hostile_lane.affected_live_view_count, 0);
}
