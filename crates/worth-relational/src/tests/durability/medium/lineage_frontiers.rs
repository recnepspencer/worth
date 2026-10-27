//! D7 protects durable identities; unacknowledged issuance is process-local.
use crate::durability::data::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
use crate::tests::support::*;

#[derive(Clone, Copy, Debug)]
enum RestoreLane {
    Image,
    Embedded,
    Cold,
}

fn discard_after_a_durable_anchor(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
) -> (u64, u64) {
    create_entity(execution, runtime, "published-lineage-anchor");
    let durable_floor = runtime.lineage.identity_frontiers();
    let position = runtime.history().latest_patch_stream_position();
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create("abandoned-lineage-candidate"),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let spent = runtime.lineage.identity_frontiers();
    assert!(spent.0 > durable_floor.0 && spent.1 > durable_floor.1);
    runtime
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .unwrap();
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    assert_eq!(
        runtime.lineage.identity_frontiers(),
        spent,
        "live discard never rewinds issuance"
    );
    durable_floor
}

#[test]
fn checkpoint_restore_protects_published_lineage_identities_in_each_lane() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    for lane in [RestoreLane::Image, RestoreLane::Embedded, RestoreLane::Cold] {
        let producer = if matches!(lane, RestoreLane::Image) {
            runtime_with_test_schema()
        } else {
            durable_runtime_with_test_schema()
        };
        let durable_floor = discard_after_a_durable_anchor(execution, &producer);
        let mut receiver = if matches!(lane, RestoreLane::Image) {
            runtime_with_test_schema()
        } else {
            durable_runtime_with_test_schema()
        };
        let plan = if matches!(lane, RestoreLane::Image) {
            producer
                .durability_authority()
                .checkpoint(execution)
                .unwrap();
            producer
                .durability()
                .recovery_plan(
                    execution,
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .unwrap()
        } else {
            let placement = if matches!(lane, RestoreLane::Cold) {
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                }
            } else {
                RelationalCheckpointArtifactPolicy::Embedded
            };
            let native = producer
                .durability_authority()
                .native_checkpoint_with_policy(execution.allocation_policy(), placement)
                .unwrap();
            attached_test_medium(&producer).file_checkpoint(native);
            medium_recovery_plan(
                execution,
                &producer,
                &mut receiver,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap()
        };
        if matches!(lane, RestoreLane::Cold) {
            let checkpoint = plan.checkpoint.as_ref().unwrap();
            assert!(checkpoint.directory.is_some());
            assert!(
                checkpoint.lineage.nodes().is_empty(),
                "cold reopen loads no node payloads"
            );
        }
        receiver
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        assert_eq!(
            receiver.lineage.identity_frontiers(),
            durable_floor,
            "{lane:?}: D7 resumes one past durable values"
        );
        let created = create_entity_outcome(execution, &receiver, "after-lineage-reopen");
        let envelope = receiver
            .replay()
            .canonical_commit_envelope(execution, created.commit.commit_id)
            .unwrap()
            .map(|envelope| envelope.as_ref().clone())
            .unwrap();
        assert!(!envelope.lineage_events().is_empty());
        for event in envelope.lineage_events() {
            assert!(
                event.event_id() >= durable_floor.1,
                "{lane:?}: a published event id must never return"
            );
            assert!(
                event.targets().iter().all(|id| id.0 >= durable_floor.0),
                "{lane:?}: a published node id must never return"
            );
        }
    }
}

#[test]
fn inflight_tail_replay_resumes_after_the_largest_durable_lineage_values() {
    let backing = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    let producer = durable_runtime_with_test_schema();
    create_entity(execution, &producer, "lineage-tail-anchor");
    let prepare = |name| {
        let mut transaction = test_owner_begin_transaction_for_main(&producer);
        transaction
            .push_batch(batch_create(name), execution.allocation_policy())
            .unwrap();
        producer
            .prepare_branch_transaction(execution, transaction)
            .unwrap()
    };
    let inflight = prepare("lineage-inflight-tail");
    let abandoned = prepare("lineage-later-abandoned");
    producer
        .preparation_port()
        .discard_prepared_candidate(abandoned)
        .unwrap();
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    attached_test_medium(&producer).file_checkpoint(native);
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = producer
        .publication_port()
        .compare_and_publish(execution, inflight)
    else {
        panic!("the checkpoint preserves the prepared candidate's publication basis");
    };
    let committed = producer.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&producer, &committed);
    let envelope = producer
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .unwrap();
    let node_floor = envelope
        .lineage_events()
        .iter()
        .flat_map(|event| event.targets())
        .map(|id| id.0)
        .max()
        .unwrap()
        + 1;
    let event_floor = envelope
        .lineage_events()
        .iter()
        .map(|event| event.event_id())
        .max()
        .unwrap()
        + 1;
    let mut receiver = durable_runtime_with_test_schema();
    let plan = medium_recovery_plan(
        execution,
        &producer,
        &mut receiver,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    assert_eq!(
        plan.tail_log.len(),
        1,
        "the inflight publication belongs to the tail"
    );
    receiver
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        receiver.lineage.identity_frontiers(),
        (node_floor, event_floor)
    );
    let created = create_entity_outcome(execution, &receiver, "lineage-after-inflight-replay");
    let envelope = receiver
        .replay()
        .canonical_commit_envelope(execution, created.commit.commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .unwrap();
    assert!(envelope
        .lineage_events()
        .iter()
        .all(|event| event.event_id() >= event_floor
            && event.targets().iter().all(|id| id.0 >= node_floor)));
}
