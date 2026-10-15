use super::RuntimeApiStabilizationAdapter;
use crate::harness::certification::{
    unmet_required_assertion_classes, HostileExpectation, ParityAnchor, RequiredAssertionClass,
};
use crate::runtime::{WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupportStatus};

#[test]
fn runtime_api_stabilization_rows_have_required_outputs_and_meaningful_assertions() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_test(
        request_execution,
    );

    for row in &matrix.rows {
        for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
            assert!(
                lane.support_gated_neighbor_denial_count >= 1,
                "golden DX rows must prove future-neighbor denials"
            );
            assert!(
                lane.delivery_residue_count >= 1,
                "golden DX rows must prove delivery or pending-intent residue"
            );
            assert_ne!(
                lane.public_api_naming_contract_digest, lane.public_api_surface_digest,
                "naming contract must be distinct from backend family support posture"
            );
            assert!(lane.stable_family_count >= 10);
            assert!(lane.deferred_family_count >= 2);
        }
    }
}

#[test]
fn runtime_api_stabilization_golden_transcripts_are_executable_and_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_test(
        request_execution,
    );
    let mut digests = matrix
        .rows
        .iter()
        .map(|row| {
            assert!(
                row.control_lane.executable_transcript_digest
                    != row.control_lane.golden_transcript_digest,
                "row '{}' should carry runtime-executed evidence beyond label evidence",
                row.row_name
            );
            (
                row.row_name,
                row.control_lane.executable_transcript_digest.clone(),
            )
        })
        .collect::<Vec<_>>();
    digests.sort_by(|left, right| left.1.cmp(&right.1));
    for pair in digests.windows(2) {
        assert_ne!(
            pair[0].1, pair[1].1,
            "rows '{}' and '{}' should not collapse to the same executable transcript digest",
            pair[0].0, pair[1].0
        );
    }
}

#[test]
fn runtime_api_stabilization_rows_enforce_required_assertion_classes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_test(
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
        assert_eq!(
            row.hostile_lane.status,
            WorthQueryRuntimeFamilySupportStatus::DeferredDebt
        );
        assert!(row
            .hostile_lane
            .counter_snapshot
            .contains("authority_residue=0"));
        covered.push(RequiredAssertionClass::ZeroResidue);
    }

    covered.sort();
    covered.dedup();
    let required = [
        RequiredAssertionClass::Equality,
        RequiredAssertionClass::TypedFailure,
        RequiredAssertionClass::ZeroResidue,
    ];
    let missing = unmet_required_assertion_classes(&covered, &required);
    assert!(missing.is_empty(), "missing assertion classes: {missing:?}");
}

#[test]
fn runtime_api_stabilization_deferred_gates_name_future_owners() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_test(
        request_execution,
    );
    let expected = [
        (
            "store-backed-parity-deferred-gate",
            WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
        ),
        (
            "durable-restart-deferred-gate",
            WorthQueryRuntimeFacadeFamily::DurableArtifacts,
        ),
    ];

    for (row_name, family) in expected {
        let row = matrix
            .rejection_rows
            .iter()
            .find(|row| row.row_name == row_name)
            .expect("deferred gate row should exist");
        assert_eq!(row.hostile_lane.family, family);
        assert_eq!(
            row.hostile_lane.status,
            WorthQueryRuntimeFamilySupportStatus::DeferredDebt
        );
        assert_ne!(
            row.hostile_lane.deferred_temporal_async_gate_digest,
            row.hostile_lane.failure_digest
        );
    }
}
