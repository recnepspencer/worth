use super::matrices::PolicyCertificationRow;
use super::{execute_policy_request, PolicyHarnessExecution, PolicyHarnessTarget};
use crate::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeMappingId,
    BridgeMappingRegistration, BridgePolicyRejectionKind, BridgePolicyRejectionStage,
    BridgeProducerMetadata, BridgeRuntimePolicy, BridgeTruthViewPolicyResolution,
    CoarseRoutingMode, MappingSelector, SignalInvalidationScope, SnapshotReadRecord,
    SnapshotReadRequest, TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope,
    TruthSnapshotIdentity,
};
use crate::harness::fixtures::{
    BridgeHarnessFixture, InMemoryRelationalBridgeSource, RecordingSignalBridgeSink,
    SnapshotFixture,
};

fn runtime_with_policy(policy: BridgeRuntimePolicy) -> crate::facade::RuntimeBridge {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid policy fixture field key"),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(policy)
        .register_mapping(registration())
        .build()
        .expect("policy certification runtime should build")
}

fn fixture_with_policy(policy: BridgeRuntimePolicy) -> BridgeHarnessFixture {
    BridgeHarnessFixture::new(vec![registration()])
        .with_policy(policy)
        .with_committed_patch(committed_patch(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            worth_foundational::facade::FieldKey::new("name".to_owned())
                .expect("valid policy fixture field key"),
        ))
        .with_snapshot(snapshot(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            "alice",
        ))
}

fn registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("profile-name"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("user"),
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::FieldKey::new("name".to_owned())
                .expect("valid native field key"),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned("signal.profile"),
        CoarseRoutingMode::Direct,
    )
}

fn committed_patch(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    field_key: worth_foundational::facade::FieldKey,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            commit_identity,
            patch_identity,
            snapshot_identity,
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
        ),
        vec![BridgeCommittedPatchItem::with_target(
            "user",
            crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid bridge patch aspect key"),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(field_key),
            ),
        )],
    )
    .expect("policy certification committed patch should construct")
}

fn snapshot(snapshot_identity: TruthSnapshotIdentity, value: &str) -> SnapshotFixture {
    SnapshotFixture::new(
        snapshot_identity,
        vec![SnapshotReadRecord::for_request(
            &SnapshotReadRequest::for_coarse(
                "user",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid policy snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            ),
            worth_foundational::facade::AspectValue::String(value.into()),
        )],
    )
}

#[test]
fn provenance_certification_retains_typed_policy_route_and_counter_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_policy(BridgeRuntimePolicy::development());
    let fixture = fixture_with_policy(BridgeRuntimePolicy::development());
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

    let runtime = runtime_with_policy(BridgeRuntimePolicy::development());
    let fixture = fixture_with_policy(BridgeRuntimePolicy::development());
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

    let runtime = runtime_with_policy(BridgeRuntimePolicy::development());
    let fixture = fixture_with_policy(BridgeRuntimePolicy::development());
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
