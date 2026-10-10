use super::{
    execute_stream_request, NativeStreamCommitWindow, StreamHarnessExecution, StreamHarnessTarget,
};
use crate::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeMappingId,
    BridgeMappingRegistration, BridgeProducerMetadata, CoarseRoutingMode, MappingSelector,
    RuntimeBridgeBuilder, SignalInvalidationScope, SnapshotReadRecord, SnapshotReadRequest,
    TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope, TruthSnapshotIdentity,
};
use crate::harness::fixtures::{
    InMemoryRelationalBridgeSource, RecordingSignalBridgeSink, SnapshotFixture,
};
use crate::stream::BackpressureDecisionRecord;

fn runtime_with_stream_source() -> crate::facade::RuntimeBridge {
    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
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
        .register_mapping(registration())
        .build()
        .expect("stream certification runtime should build")
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
    .expect("stream certification committed patch should construct")
}

fn snapshot(snapshot_identity: TruthSnapshotIdentity, text: &str) -> SnapshotFixture {
    SnapshotFixture::new(
        snapshot_identity,
        vec![SnapshotReadRecord::for_request(
            &SnapshotReadRequest::for_coarse(
                "user",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid stream snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            ),
            worth_foundational::facade::AspectValue::String(text.into()),
        )],
    )
}

fn native_stream_window() -> NativeStreamCommitWindow {
    NativeStreamCommitWindow::from_commits([
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
    ])
    .expect("native stream commit window should construct")
}

#[test]
fn native_window_rejects_empty_stream_window_before_execution() {
    let error = NativeStreamCommitWindow::from_commits([])
        .expect_err("empty native stream window must be rejected");

    assert!(
        error.to_string().contains("at least one commit identity"),
        "unexpected stream window construction error: {error}"
    );
}

#[test]
fn routing_and_replay_audit_executions_retain_typed_stream_member_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_stream_source();
    let routing_execution = execute_stream_request(
        &runtime,
        StreamHarnessTarget::RoutingWindow {
            window: native_stream_window(),
        },
        resource_request,
    )
    .expect("routing stream execution should succeed");
    let replay_execution = execute_stream_request(
        &runtime,
        StreamHarnessTarget::ReplayAuditWindow {
            window: native_stream_window(),
        },
        resource_request,
    )
    .expect("replay stream execution should succeed");

    let StreamHarnessExecution::Routing {
        window: routing_window,
        result: routing_result,
        checkpoint,
        replay_record,
    } = routing_execution
    else {
        panic!("expected routing stream execution");
    };
    let StreamHarnessExecution::ReplayAudit {
        window: replay_window,
        result: replay_result,
    } = replay_execution
    else {
        panic!("expected replay-audit stream execution");
    };

    assert_eq!(routing_window.members().len(), 2);
    assert_eq!(replay_window.members().len(), 2);
    assert_eq!(
        routing_window
            .members()
            .first()
            .map(|member| member.stream_member_identity()),
        replay_window
            .members()
            .first()
            .map(|member| member.stream_member_identity())
    );
    assert_eq!(
        routing_window
            .members()
            .last()
            .map(|member| member.stream_member_identity()),
        replay_window
            .members()
            .last()
            .map(|member| member.stream_member_identity())
    );
    assert_eq!(
        routing_result.summary().stream_digest(),
        replay_result.summary().stream_digest()
    );
    assert_eq!(
        routing_window.member_set_digest(),
        replay_window.member_set_digest()
    );
    assert_ne!(
        routing_result.summary().window_digest(),
        replay_result.summary().window_digest()
    );
    assert_eq!(routing_result.summary().counters().stream_member_count(), 2);
    assert_eq!(replay_result.summary().counters().stream_replay_count(), 1);
    assert_eq!(checkpoint.checkpoint_member_count(), 2);
    assert_eq!(
        replay_record.stream_window_identity(),
        routing_window.stream_window_identity()
    );
}

#[test]
fn replay_audit_execution_is_deterministic_from_typed_records() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let first_runtime = runtime_with_stream_source();
    let second_runtime = runtime_with_stream_source();
    let first_execution = execute_stream_request(
        &first_runtime,
        StreamHarnessTarget::ReplayAuditWindow {
            window: native_stream_window(),
        },
        resource_request,
    )
    .expect("first replay stream execution should succeed");
    let second_execution = execute_stream_request(
        &second_runtime,
        StreamHarnessTarget::ReplayAuditWindow {
            window: native_stream_window(),
        },
        resource_request,
    )
    .expect("second replay stream execution should succeed");

    let StreamHarnessExecution::ReplayAudit {
        window: first_window,
        result: first_result,
    } = first_execution
    else {
        panic!("expected first replay-audit stream execution");
    };
    let StreamHarnessExecution::ReplayAudit {
        window: second_window,
        result: second_result,
    } = second_execution
    else {
        panic!("expected second replay-audit stream execution");
    };

    assert_eq!(first_window.digest(), second_window.digest());
    assert_eq!(
        first_result.summary().stream_digest(),
        second_result.summary().stream_digest()
    );
    assert_eq!(
        first_result.checkpoint().checkpoint_token_identity(),
        second_result.checkpoint().checkpoint_token_identity()
    );
    assert_eq!(
        first_result.replay_record().digest(),
        second_result.replay_record().digest()
    );
}

#[test]
fn routing_execution_retains_typed_pressure_and_counter_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_stream_source();
    let execution = execute_stream_request(
        &runtime,
        StreamHarnessTarget::RoutingWindow {
            window: native_stream_window(),
        },
        resource_request,
    )
    .expect("routing stream execution should succeed");

    let StreamHarnessExecution::Routing { window, result, .. } = execution else {
        panic!("expected routing stream execution");
    };
    let pressure = BackpressureDecisionRecord::classify(&window);

    assert_eq!(pressure.pressure_class(), "elevated-pressure");
    assert_eq!(pressure.counters().stream_backpressure_signal_count(), 1);
    assert_eq!(result.summary().delivered_member_count(), 2);
    assert_eq!(result.summary().delivered_route_count(), 2);
    assert_eq!(result.summary().counters().stream_window_count(), 1);
    assert_eq!(result.summary().counters().stream_checkpoint_count(), 0);
}
