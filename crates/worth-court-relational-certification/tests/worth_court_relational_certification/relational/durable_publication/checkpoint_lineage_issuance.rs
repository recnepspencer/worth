//! Published lineage identities remain spent across each real restoration lane.
use super::branch_history_publications::transaction;
use super::checkpoint_lineage_publication::published_floor;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy},
    history::BranchId,
    mvcc::RelationalPublicationOutcome,
    runtime::RelationalRuntime,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    durable_medium::TestDurableMedium,
    durable_publication::{batch_create, create_entity_outcome, durable_runtime, memory_runtime},
};
#[derive(Clone, Copy, Debug)]
enum RestoreLane {
    Image,
    Embedded,
    Cold,
}
fn discard_after_anchor(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> (u64, u64) {
    let committed = create_entity_outcome(execution, runtime, "published-lineage-anchor");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .unwrap();
    let floor = published_floor(&envelope);
    let position = runtime.publication().latest_patch().unwrap().position;
    release_test_commit_snapshot(runtime, &committed);
    let mut txn = transaction(runtime, &BranchId("main".into()));
    txn.push_batch(
        batch_create("abandoned-lineage-candidate"),
        execution.allocation_policy(),
    )
    .unwrap();
    let proposal = txn.validate(execution, runtime).unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    runtime
        .owner_component_services()
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .unwrap();
    assert_eq!(
        runtime.publication().latest_patch().unwrap().position,
        position
    );
    floor
}
#[test]
fn published_lineage_identities_stay_spent_after_image_embedded_and_cold_recovery() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for lane in [RestoreLane::Image, RestoreLane::Embedded, RestoreLane::Cold] {
        let medium = TestDurableMedium::new();
        let producer = if matches!(lane, RestoreLane::Image) {
            memory_runtime()
        } else {
            durable_runtime(&medium)
        };
        let floor = discard_after_anchor(execution, &producer);
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
                RelationalCheckpointArtifactPolicy::Embedded
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
        receiver
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        let created = create_entity_outcome(execution, &receiver, "after-lineage-reopen");
        let envelope = receiver
            .replay()
            .canonical_commit_envelope(execution, created.commit.commit_id)
            .unwrap()
            .unwrap();
        assert!(!envelope.lineage_events().is_empty());
        for event in envelope.lineage_events() {
            assert!(
                event.event_id() >= floor.1,
                "{lane:?}: a published event id must never return"
            );
            assert!(
                event.targets().iter().all(|id| id.0 >= floor.0),
                "{lane:?}: a published node id must never return"
            );
        }
    }
}
#[test]
fn inflight_lineage_tail_recovery_keeps_every_published_identity_spent() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let anchor = create_entity_outcome(execution, &producer, "lineage-tail-anchor");
    release_test_commit_snapshot(&producer, &anchor);
    let prepare = |name| {
        let mut txn = transaction(&producer, &BranchId("main".into()));
        txn.push_batch(batch_create(name), execution.allocation_policy())
            .unwrap();
        let proposal = txn.validate(execution, &producer).unwrap();
        producer
            .prepare_validated_proposal(execution, proposal)
            .unwrap()
    };
    let inflight = prepare("lineage-inflight-tail");
    let abandoned = prepare("lineage-later-abandoned");
    producer
        .owner_component_services()
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
        panic!("the checkpoint preserves the prepared publication basis")
    };
    let committed = producer.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&producer, &committed);
    let envelope = producer
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .unwrap();
    let (node_floor, event_floor) = published_floor(&envelope);
    let mut receiver = durable_runtime(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let outcome = receiver
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        outcome.coverage.replayed_tail_commits, 1,
        "the inflight publication belongs to the tail"
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
