use super::matrices::PolicyCertificationRow;
use super::{execute_policy_request, PolicyHarnessExecution, PolicyHarnessTarget};
use crate::facade::{
    BridgePolicyRejectionKind, BridgePolicyRejectionStage, BridgeRuntimePolicy,
    BridgeTruthViewPolicyResolution,
};
use crate::harness::fixtures::{
    BridgeHarnessFixture, NamedTruth, RecordingSignalBridgeSink, ScenarioRelationalTruth,
};

fn runtime_and_fixture() -> (crate::facade::RuntimeBridge, BridgeHarnessFixture) {
    let policy = BridgeRuntimePolicy::development();
    let (truth, _commit) = ScenarioRelationalTruth::named("alice", false);
    let source = truth.native.source.clone();
    let runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(policy)
        .register_mapping(NamedTruth::mapping())
        .build()
        .expect("policy certification runtime should build");
    (
        runtime,
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()])
            .with_policy(policy)
            .with_relational_truth(truth),
    )
}

#[test]
fn provenance_certification_retains_typed_policy_route_and_counter_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, fixture) = runtime_and_fixture();
    let execution = execute_policy_request(
        &runtime,
        &fixture,
        PolicyHarnessTarget::ProvenanceCertification,
        resource_request,
    )
    .expect("provenance certification should execute");

    let PolicyHarnessExecution::Provenance {
        policy_matrix,
        policy_provenance_report,
        request_policy_matrix,
        route_policy_matrix,
        routing_digest,
        counter_snapshot,
        ..
    } = execution
    else {
        panic!("expected provenance certification");
    };

    assert_eq!(policy_matrix.rows().len(), 2);
    assert!(policy_matrix
        .rows()
        .iter()
        .all(|row| matches!(row, PolicyCertificationRow::Admitted(_))));
    assert!(matches!(
        &policy_matrix.rows()[0],
        PolicyCertificationRow::Admitted(row)
            if row.declaration_identity()
                == &crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                    "policy-cert:deterministic-authoritative",
                )
    ));
    assert!(matches!(
        &policy_matrix.rows()[1],
        PolicyCertificationRow::Admitted(row)
            if row.declaration_identity()
                == &crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                    "policy-cert:optimized-preview",
                )
    ));
    assert_eq!(policy_provenance_report.rows().len(), 2);
    assert_eq!(request_policy_matrix.rows().len(), 2);
    assert_eq!(route_policy_matrix.rows().len(), 2);
    assert_ne!(
        route_policy_matrix.rows()[0].lowered_policy_identity(),
        route_policy_matrix.rows()[1].lowered_policy_identity()
    );
    assert!(route_policy_matrix.rows()[0]
        .lowered_policy_identity()
        .as_str()
        .starts_with("lowered-bridge-execution-policy:sha256:"));
    assert!(route_policy_matrix.rows()[1]
        .lowered_policy_identity()
        .as_str()
        .starts_with("lowered-bridge-execution-policy:sha256:"));
    assert!(routing_digest.is_some());
    assert_eq!(counter_snapshot.declaration_count(), 2);
    assert_eq!(counter_snapshot.declaration_width_count(), 8);
    assert_eq!(counter_snapshot.admission_width_count(), 8);
    assert_eq!(counter_snapshot.replay_bundle_count(), 2);
    assert_eq!(counter_snapshot.ambient_policy_leak_count(), 0);
}

#[test]
fn rejection_certification_retains_typed_rejection_rows_and_zero_authority_escape_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, fixture) = runtime_and_fixture();
    let execution = execute_policy_request(
        &runtime,
        &fixture,
        PolicyHarnessTarget::RejectionCertification,
        resource_request,
    )
    .expect("rejection certification should execute");

    let PolicyHarnessExecution::Rejection {
        policy_matrix,
        counter_snapshot,
        ..
    } = execution
    else {
        panic!("expected rejection certification");
    };
    let rows = policy_matrix.rows();

    assert_eq!(rows.len(), 2);
    assert_eq!(counter_snapshot.admitted_contract_count(), 0);
    assert_eq!(counter_snapshot.rejected_contract_count(), 2);
    assert_eq!(counter_snapshot.substantive_illegality_count(), 2);
    assert_eq!(counter_snapshot.authority_escape_count(), 0);
    assert!(matches!(
        &rows[0],
        PolicyCertificationRow::Rejection(row)
            if row.failure_kind() == BridgePolicyRejectionKind::UnsupportedExecutionMode
                && row.stage() == BridgePolicyRejectionStage::Validation
                && row.declaration_identity()
                    == &crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                        "policy-cert:rejection-optimized-authoritative",
                    )
    ));
    assert!(matches!(
        &rows[1],
        PolicyCertificationRow::Rejection(row)
            if row.failure_kind() == BridgePolicyRejectionKind::ReplayPolicyConflict
                && row.stage() == BridgePolicyRejectionStage::Admission
                && row.declaration_identity()
                    == &crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                        "policy-cert:rejection-replay-conflict",
                    )
    ));
}

#[test]
fn ambient_leak_certification_retains_typed_request_equivalence_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, fixture) = runtime_and_fixture();
    let execution = execute_policy_request(
        &runtime,
        &fixture,
        PolicyHarnessTarget::AmbientLeakCertification,
        resource_request,
    )
    .expect("ambient leak certification should execute");

    let PolicyHarnessExecution::AmbientLeak {
        policy_matrix,
        policy_provenance_report,
        request_policy_matrix,
        counter_snapshot,
        ..
    } = execution
    else {
        panic!("expected ambient leak certification");
    };
    let request_rows = request_policy_matrix.rows();

    assert_eq!(policy_matrix.rows().len(), 3);
    assert_eq!(policy_provenance_report.rows().len(), 3);
    assert_eq!(request_rows.len(), 3);
    assert!(matches!(
        request_policy_matrix.branch_local_resolution(),
        Some(&BridgeTruthViewPolicyResolution::Admitted(_))
    ));
    assert!(matches!(
        request_policy_matrix.historical_resolution(),
        Some(&BridgeTruthViewPolicyResolution::Admitted(_))
    ));
    assert_eq!(
        request_rows[0].provenance_row().semantic_policy_digest(),
        request_rows[2].provenance_row().semantic_policy_digest()
    );
    assert_eq!(
        request_rows[0].semantic_route_planning_policy_digest(),
        request_rows[2].semantic_route_planning_policy_digest()
    );
    assert_ne!(
        request_rows[0].provenance_row().semantic_policy_digest(),
        request_rows[1].provenance_row().semantic_policy_digest()
    );
    assert_eq!(counter_snapshot.policy_request_count(), 3);
    assert_eq!(counter_snapshot.truth_view_interleave_count(), 2);
    assert_eq!(counter_snapshot.ambient_policy_leak_count(), 0);
}
