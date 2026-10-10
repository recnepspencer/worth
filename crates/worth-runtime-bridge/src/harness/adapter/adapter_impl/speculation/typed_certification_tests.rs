use super::{execute_speculation_request, SpeculationHarnessExecution, SpeculationHarnessTarget};
use crate::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeMappingId,
    BridgeMappingRegistration, BridgePreviewLifecycleStateKind, BridgeProducerMetadata,
    BridgeRuntimePolicy, CoarseRoutingMode, MappingSelector, RuntimeBridgeBuilder,
    SignalInvalidationScope, SnapshotReadRecord, SnapshotReadRequest, TruthPatchIdentity,
    TruthPatchScope, TruthSnapshotIdentity,
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
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_policy(policy)
        .register_mapping(registration())
        .build()
        .expect("speculation certification runtime should build")
}

fn fixture_with_policy(policy: BridgeRuntimePolicy) -> BridgeHarnessFixture {
    BridgeHarnessFixture::new(vec![registration()])
        .with_policy(policy)
        .with_committed_patch(committed_patch(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
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
    commit_identity: crate::facade::TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
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
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("name".to_owned())
                        .expect("valid foundational field key"),
                ),
            ),
        )],
    )
    .expect("speculation certification committed patch should construct")
}

fn snapshot(snapshot_identity: TruthSnapshotIdentity, text: &str) -> SnapshotFixture {
    SnapshotFixture::new(
        snapshot_identity,
        vec![SnapshotReadRecord::for_request(
            &SnapshotReadRequest::for_coarse(
                "user",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid speculation snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            ),
            worth_foundational::facade::AspectValue::String(text.into()),
        )],
    )
}

#[test]
fn discard_certification_retains_typed_residue_counter_and_route_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_policy(BridgeRuntimePolicy::development());
    let fixture = fixture_with_policy(BridgeRuntimePolicy::development());
    let execution = execute_speculation_request(
        &runtime,
        &fixture,
        SpeculationHarnessTarget::DiscardCertification,
        resource_request,
    )
    .expect("discard certification should execute");

    let SpeculationHarnessExecution::Discard {
        execution_record,
        discard_record,
        routing_digest,
    } = execution
    else {
        panic!("expected discard certification");
    };

    assert_eq!(
        discard_record.preview_execution_record_identity(),
        execution_record.record_identity()
    );
    assert_eq!(
        discard_record
            .residue_report()
            .authoritative_residue_count(),
        0
    );
    assert_eq!(discard_record.counters().preview_session_count_touched(), 1);
    assert_eq!(discard_record.counters().discard_artifact_count(), 2);
    assert_eq!(
        discard_record
            .counters()
            .retained_non_authoritative_artifact_count(),
        2
    );
    assert!(routing_digest.is_some());
}

#[test]
fn promotion_certification_retains_typed_commit_replay_and_discard_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_policy(BridgeRuntimePolicy::development());
    let fixture = fixture_with_policy(BridgeRuntimePolicy::development());
    let execution = execute_speculation_request(
        &runtime,
        &fixture,
        SpeculationHarnessTarget::PromotionCertification,
        resource_request,
    )
    .expect("promotion certification should execute");

    let SpeculationHarnessExecution::Promotion {
        promoted_execution_record,
        promotion_record,
        promoted_replay_bundle,
        discarded_execution_record,
        discarded_record,
        discarded_replay_bundle,
        routing_digest,
        diagnostics_digest,
    } = execution
    else {
        panic!("expected promotion certification");
    };

    assert_eq!(
        promotion_record.preview_execution_record_identity(),
        promoted_execution_record.record_identity()
    );
    assert_eq!(
        discarded_record.preview_execution_record_identity(),
        discarded_execution_record.record_identity()
    );
    assert_eq!(
        promoted_replay_bundle.lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Promoted
    );
    assert_eq!(
        discarded_replay_bundle.lifecycle_outcome(),
        BridgePreviewLifecycleStateKind::Discarded
    );
    assert_eq!(promotion_record.counters().promotion_proof_checks(), 1);
    assert_eq!(
        discarded_record
            .residue_report()
            .authoritative_residue_count(),
        0
    );
    assert!(routing_digest.is_some());
    assert!(diagnostics_digest.starts_with("speculation-diagnostics-digest:"));
}

#[test]
fn churn_certification_retains_typed_branch_isolation_and_resource_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_policy(BridgeRuntimePolicy::development());
    let fixture = fixture_with_policy(BridgeRuntimePolicy::development());
    let execution = execute_speculation_request(
        &runtime,
        &fixture,
        SpeculationHarnessTarget::ChurnCertification,
        resource_request,
    )
    .expect("churn certification should execute");

    let SpeculationHarnessExecution::Churn { certification } = execution else {
        panic!("expected churn certification");
    };
    let branch_matrix = certification.branch_isolation_matrix();
    let resource_report = certification.resource_bound_report();
    let counter_snapshot = certification.counter_snapshot();

    assert_eq!(
        certification
            .preview_replay_bundle_set()
            .replay_bundles()
            .len(),
        3
    );
    assert!(certification
        .preview_replay_bundle_set()
        .replay_bundles()
        .iter()
        .all(|bundle| bundle.lifecycle_outcome() == BridgePreviewLifecycleStateKind::Discarded));
    assert_eq!(branch_matrix.rows().len(), 3);
    assert_eq!(
        branch_matrix.baseline_authoritative_route_digest(),
        branch_matrix.final_authoritative_route_digest()
    );
    assert!(branch_matrix
        .rows()
        .iter()
        .all(|row| row.authoritative_route_digest_after_discard()
            == branch_matrix.baseline_authoritative_route_digest()));
    assert!(branch_matrix
        .rows()
        .iter()
        .all(|row| row.lifecycle_outcome() == BridgePreviewLifecycleStateKind::Discarded));
    assert_eq!(resource_report.retained_preview_execution_record_count(), 3);
    assert_eq!(resource_report.retained_preview_discard_record_count(), 3);
    assert_eq!(resource_report.retained_preview_promotion_record_count(), 0);
    assert_eq!(resource_report.authoritative_route_observation_count(), 5);
    assert_eq!(counter_snapshot.preview_session_count_touched(), 3);
    assert_eq!(counter_snapshot.authoritative_route_observation_count(), 5);
}
