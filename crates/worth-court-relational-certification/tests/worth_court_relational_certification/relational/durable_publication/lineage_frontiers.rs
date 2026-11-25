//! D7 protects durable lineage identities; unacknowledged issuance is process-local.
//! Restored from f9a53a5d17:tests/durability/medium/lineage_frontiers.rs over
//! TestDurableMedium using the owner's read-only lineage residency observation.
use super::*;
use worth_relational::facade::{
    durability::RelationalCheckpointArtifactPolicy, runtime::RelationalRuntime,
};

#[derive(Clone, Copy, Debug)]
enum RestoreLane {
    Image,
    Embedded,
    Cold,
}

fn discard_after_a_durable_anchor(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> (u64, u64) {
    create_entity(execution, runtime, "published-lineage-anchor");
    let obs = runtime.observe_lineage_residency();
    let durable_floor = (obs.node_identity_frontier, obs.event_identity_frontier);
    let position = runtime.history().latest_patch_stream_position();
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            batch_create("abandoned-lineage-candidate"),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let spent_obs = runtime.observe_lineage_residency();
    let spent = (
        spent_obs.node_identity_frontier,
        spent_obs.event_identity_frontier,
    );
    assert!(spent.0 > durable_floor.0 && spent.1 > durable_floor.1);
    runtime
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .unwrap();
    assert_eq!(runtime.history().latest_patch_stream_position(), position);
    let after = runtime.observe_lineage_residency();
    assert_eq!(
        (after.node_identity_frontier, after.event_identity_frontier),
        spent,
        "live discard never rewinds issuance"
    );
    durable_floor
}

#[test]
fn checkpoint_restore_protects_published_lineage_identities_in_each_lane() {
    let backing =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(
        );
    let execution = &ExecutionRequest::serial(&backing);
    for lane in [RestoreLane::Image, RestoreLane::Embedded, RestoreLane::Cold] {
        let medium = TestDurableMedium::new();
        let producer = if matches!(lane, RestoreLane::Image) {
            memory_runtime()
        } else {
            durable_runtime(&medium)
        };
        let durable_floor = discard_after_a_durable_anchor(execution, &producer);
        let mut receiver = if matches!(lane, RestoreLane::Image) {
            memory_runtime()
        } else {
            durable_runtime(&medium)
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
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                }
            };
            let native = producer
                .durability_authority()
                .native_checkpoint_with_policy(execution, execution.allocation_policy(), placement)
                .unwrap();
            medium.file_checkpoint(native);
            receiver
                .durability()
                .recovery_plan_from_source(
                    execution,
                    medium.recovery_source(),
                    RecoveryVerificationMode::NormalRecoveryVerification,
                )
                .unwrap()
        };
        if matches!(lane, RestoreLane::Cold) {
            let checkpoint = plan.checkpoint.as_ref().unwrap();
            assert!(checkpoint.has_cold_directory());
            assert!(
                checkpoint.lineage.nodes().is_empty(),
                "cold reopen loads no node payloads"
            );
        }
        receiver
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        let obs = receiver.observe_lineage_residency();
        assert_eq!(
            (obs.node_identity_frontier, obs.event_identity_frontier),
            durable_floor,
            "{lane:?}: D7 resumes one past durable values"
        );
        if matches!(lane, RestoreLane::Cold) {
            assert_eq!(obs.resident_node_count, 0);
        }
        let created = create_entity_outcome(execution, &receiver, "after-lineage-reopen");
        let envelope = receiver
            .replay()
            .canonical_commit_envelope(execution, created.commit.commit_id)
            .unwrap()
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
    let backing =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request(
        );
    let execution = &ExecutionRequest::serial(&backing);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    create_entity(execution, &producer, "lineage-tail-anchor");
    let prepare = |name| {
        let mut transaction = begin_main_transaction(&producer);
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
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    medium.file_checkpoint(native);
    let RelationalPublicationOutcome::Performed(performed) = producer
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
    let mut receiver = durable_runtime(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    assert_eq!(
        plan.tail_log_len(),
        1,
        "the inflight publication belongs to the tail"
    );
    receiver
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let obs = receiver.observe_lineage_residency();
    assert_eq!(
        (obs.node_identity_frontier, obs.event_identity_frontier),
        (node_floor, event_floor)
    );
    let created = create_entity_outcome(execution, &receiver, "lineage-after-inflight-replay");
    let envelope = receiver
        .replay()
        .canonical_commit_envelope(execution, created.commit.commit_id)
        .unwrap()
        .unwrap();
    assert!(envelope
        .lineage_events()
        .iter()
        .all(|event| event.event_id() >= event_floor
            && event.targets().iter().all(|id| id.0 >= node_floor)));
}
