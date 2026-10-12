use super::{
    RuntimeApiStabilizationAdapter, RUNTIME_API_STABILIZATION_REQUIRED_CANONICAL_ROW_NAMES,
    RUNTIME_API_STABILIZATION_REQUIRED_REJECTION_ROW_NAMES,
};
use crate::harness::certification::{
    contains_row, unmet_required_assertion_classes, HostileExpectation, ParityAnchor,
    RequiredAssertionClass,
};
use crate::runtime::{WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupportStatus};

#[test]
fn runtime_api_stabilization_adapter_emits_named_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact =
        RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_artifact(
            request_execution,
        );

    assert_eq!(
        artifact.suite_name,
        "Runtime API Golden DX And Async-Safe Facade Test"
    );
    assert!(!artifact.certification_bundle_digest.is_empty());
    assert!(!artifact.coverage_matrix_digest.is_empty());
    assert!(!artifact.closeout.closeout_digest.is_empty());
}

#[test]
fn runtime_api_stabilization_matrix_covers_required_rows() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_test(
        request_execution,
    );

    for row_name in RUNTIME_API_STABILIZATION_REQUIRED_CANONICAL_ROW_NAMES {
        assert!(contains_row(&matrix, row_name), "missing {row_name}");
    }
    for row_name in RUNTIME_API_STABILIZATION_REQUIRED_REJECTION_ROW_NAMES {
        assert!(contains_row(&matrix, row_name), "missing {row_name}");
    }
}

#[test]
fn runtime_api_stabilization_rows_have_required_outputs_and_meaningful_assertions() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let matrix = RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_test(
        request_execution,
    );

    for row in &matrix.rows {
        for lane in [&row.control_lane, &row.hostile_lane, &row.parity_lane] {
            assert!(lane.has_required_outputs(), "row '{}'", row.row_name);
            assert!(lane.public_facade_only, "row '{}'", row.row_name);
            assert_eq!(
                lane.lower_runtime_plumbing_count, 0,
                "row '{}'",
                row.row_name
            );
            assert!(
                lane.meaningful_assertion_count >= 8,
                "golden DX rows must assert proof artifacts, not only compilation"
            );
            assert!(
                lane.support_gated_neighbor_denial_count >= 1,
                "golden DX rows must prove future-neighbor denials"
            );
            assert!(
                lane.delivery_residue_count >= 1,
                "golden DX rows must prove delivery or pending-intent residue"
            );
            assert!(
                lane.counter_snapshot.contains("denials=")
                    && lane.counter_snapshot.contains("residue="),
                "runtime API counter snapshot must include executable transcript proof counters"
            );
            assert!(
                lane.counter_snapshot.contains("preferred_names=")
                    && lane.counter_snapshot.contains("alternate_names="),
                "runtime API counter snapshot must include canonical naming contract counters"
            );
            assert!(
                lane.counter_snapshot.contains("support_deferred_rows=2")
                    && lane.counter_snapshot.contains("support_fail_closed=")
                    && lane.counter_snapshot.contains("parallel_api_forbidden="),
                "runtime API counter snapshot must include Phase 5 support-matrix gates"
            );
            assert!(!lane.public_api_naming_contract_digest.is_empty());
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

#[test]
fn runtime_api_stabilization_closeout_answers_required_questions() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact =
        RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_artifact(
            request_execution,
        );
    let closeout = &artifact.closeout;

    assert_eq!(
        closeout.golden_transcript_count,
        RUNTIME_API_STABILIZATION_REQUIRED_CANONICAL_ROW_NAMES.len()
    );
    assert_eq!(
        closeout.hostile_rejection_count,
        RUNTIME_API_STABILIZATION_REQUIRED_REJECTION_ROW_NAMES.len()
    );
    assert_eq!(
        closeout.lower_runtime_plumbing_count, 0,
        "closeout cannot pass if ordinary DX still reaches lower runtime plumbing"
    );
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "live"));
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "computed"));
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "branch-preview"));
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "temporal"));
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "async-resource"));
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "mixed-cause-delivery"));
    assert!(closeout
        .stable_runtime_surfaces
        .iter()
        .any(|row| row == "temporal-async-certification"));
    assert!(closeout
        .deferred_runtime_surfaces
        .iter()
        .any(|row| row == "store-backed-execution:Milestone 10"));
    assert!(closeout
        .deferred_runtime_surfaces
        .iter()
        .any(|row| row == "durable-artifacts:Milestone 11"));
    assert!(closeout
        .unsupported_runtime_surfaces
        .iter()
        .any(|row| row == "intent"));

    for required in [
        "golden transcripts are executable through the public facade",
        "support-gated future neighbors fail typed and early",
        "ordinary DX uses no lower-runtime plumbing",
        "support metadata is synchronized with admission gates",
        "handle/state/inspection contract is extension-ready",
        "store/durable behavior remains explicitly deferred",
        "downstream examples are pressure tests",
    ] {
        assert!(
            closeout
                .closeout_self_check_answers
                .iter()
                .any(|answer| answer.contains(required)),
            "closeout self-check must answer `{required}`"
        );
    }
}

#[test]
fn runtime_api_stabilization_closeout_carries_migration_and_verification_contract() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let artifact =
        RuntimeApiStabilizationAdapter::runtime_api_golden_dx_and_async_safe_facade_artifact(
            request_execution,
        );
    let closeout = &artifact.closeout;

    assert!(closeout
        .alternate_names
        .iter()
        .any(|row| row == "computed_definition=>computed"));
    assert!(closeout
        .alternate_names
        .iter()
        .any(|row| row == "preview_with_options=>preview"));
    assert!(closeout
        .alternate_names
        .iter()
        .all(|row| !row.contains("computed_declaration")));
    assert!(closeout
        .safe_to_build_now
        .iter()
        .any(|row| row.contains("handles as durable inspectable app surfaces")));
    assert!(closeout
        .safe_to_build_now
        .iter()
        .any(|row| row.contains("state/inspect access")));
    assert!(closeout
        .must_not_assume_yet
        .iter()
        .any(|row| row.contains("store-backed parity")));
    assert!(closeout
        .must_not_assume_yet
        .iter()
        .any(|row| row.contains("domain-specific")));
    assert!(closeout
        .migration_guidance
        .iter()
        .any(|row| row.contains("WorthQueryRuntime::workspace")));
    assert!(closeout
        .migration_guidance
        .iter()
        .any(|row| row.contains("extend existing handle/state/inspection contracts")));

    for command in [
        "cargo fmt -p worth-query",
        "cargo check -p worth-query --tests",
        "cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail",
        "cargo test -p worth-query",
        "cargo test -p worth-query runtime_api_stabilization",
        "cargo test -p worth-query runtime_public_support",
        "git diff --check",
    ] {
        assert!(
            closeout
                .required_verification_commands
                .iter()
                .any(|actual| actual == command),
            "closeout must require `{command}`"
        );
    }
}
