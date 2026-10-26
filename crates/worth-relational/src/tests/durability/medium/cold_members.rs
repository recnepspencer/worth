//! Faults cross the production manifest recovery and transaction read doors.
#[path = "cold_source.rs"]
mod cold_source;
use cold_source::MemberSource;
#[path = "cold_candidates.rs"]
mod cold_candidates;
#[path = "cold_current_matrix.rs"]
mod cold_current_matrix;
#[path = "cold_envelope_lifetime.rs"]
mod cold_envelope_lifetime;
#[path = "cold_envelope_matrix.rs"]
mod cold_envelope_matrix;
#[path = "cold_envelopes.rs"]
mod cold_envelopes;
#[path = "cold_index_matrix.rs"]
mod cold_index_matrix;
#[path = "cold_invariants.rs"]
mod cold_invariants;
#[path = "cold_lineage_fixture.rs"]
mod cold_lineage_fixture;
#[path = "cold_lineage_matrix.rs"]
mod cold_lineage_matrix;
#[path = "cold_lineage_nodes.rs"]
mod cold_lineage_nodes;
#[path = "cold_merges.rs"]
mod cold_merges;
#[path = "cold_observation_reads.rs"]
mod cold_observation_reads;
#[path = "cold_pins.rs"]
mod cold_pins;
#[path = "cold_read_door.rs"]
mod cold_read_door;
#[path = "cold_read_matrix.rs"]
mod cold_read_matrix;
#[path = "cold_symbols.rs"]
mod cold_symbols;
#[path = "cold_visibility.rs"]
mod cold_visibility;
#[path = "replay_pin_concurrency.rs"]
mod replay_pin_concurrency;
use crate::durability::data::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalCheckpointArtifactPolicy,
    RelationalCheckpointMember, RelationalRecoveryStop,
};
use crate::durability::medium::{
    RelationalDurableCheckpointSource, RelationalDurableMemberLocator,
    RelationalDurableRecoverySource,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

#[path = "cold_isolation.rs"]
mod isolation;

#[path = "cold_recovery_fixture.rs"]
mod cold_recovery_fixture;
use cold_recovery_fixture::{
    checkpoint_fixture, fixture, fixture_from_runtime, fixture_with_history,
};

#[test]
fn a_damaged_cold_member_is_a_sticky_typed_transaction_read_refusal() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (source, counter, entity) = fixture(execution, false);
    let mut receiving = durable_runtime_with_test_schema();
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        counter.reads.load(Ordering::Relaxed),
        0,
        "reopen must not inspect the damaged member"
    );
    let mut transaction = test_owner_begin_transaction_for_main(&receiving);
    let first = transaction.read_entity(execution, entity).unwrap_err();
    let crate::mvcc::RelationalTransactionReadDenial::StorageUnavailable(failure) = first else {
        panic!("{first:?}")
    };
    assert_eq!(
        failure.subject,
        crate::storage::data::RelationalStorageSubject::Region(entity.partition_id)
    );
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
    let reads = counter.reads.load(Ordering::Relaxed);
    counter.damaged.store(false, Ordering::Relaxed);
    let mut retry = test_owner_begin_transaction_for_main(&receiving);
    assert_eq!(
        retry.read_entity(execution, entity).unwrap_err(),
        crate::mvcc::RelationalTransactionReadDenial::StorageUnavailable(failure)
    );
    assert_eq!(
        counter.reads.load(Ordering::Relaxed),
        reads,
        "damage cannot be silently retried"
    );
    let branch = crate::history::data::BranchId("main".into());
    let before = receiving.history().branch_head(&branch);
    let mut writer = test_owner_begin_transaction_for_main(&receiving);
    writer
        .push_batch(
            WorkerIntentBatch::new("damaged-member-write").push(
                crate::transactions::data::MutationIntent::Entity(
                    crate::transactions::data::EntityMutationIntent::Delete(
                        crate::transactions::data::DeleteEntityIntent { entity_id: entity },
                    ),
                ),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let crate::transactions::data::TransactionCommitError::Preparation { error, .. } =
        writer.commit(execution, &receiving).unwrap_err()
    else {
        panic!("a cold write must refuse before publication")
    };
    assert_eq!(
        error.reason(),
        crate::transactions::data::CommitPreparationReason::StorageUnavailable(failure)
    );
    assert_eq!(receiving.history().branch_head(&branch), before);
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
}

#[test]
fn tail_replay_refuses_a_damaged_touched_member_as_cold_storage_unavailable() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (source, _, _) = fixture(execution, true);
    let mut receiving = durable_runtime_with_test_schema();
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let RelationalRecoveryStop::Durability(error) = receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err()
    else {
        panic!("durable storage refusal required")
    };
    assert_eq!(error.class, RecoveryFailureClass::ColdStorageUnavailable);
    assert_eq!(
        error.storage_unavailable.unwrap().cause,
        Cause::IntegrityMismatch
    );
    assert!(
        receiving
            .history()
            .branch_head(&crate::history::data::BranchId("main".into()))
            .is_none(),
        "replay refusal must leave the receiving runtime untouched"
    );
}

#[test]
fn a_cold_point_read_keeps_cross_partition_kind_authority_after_an_unrelated_write() {
    use crate::identity::data::PartitionId;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let source_entity =
        create_entity_in_partition(execution, &runtime, "cross-source", PartitionId(2));
    let target_entity =
        create_entity_in_partition(execution, &runtime, "cross-target", PartitionId(3));
    let relation = create_relation_outcome(
        execution,
        &runtime,
        source_entity,
        target_entity,
        "cross-edge",
    );
    release_test_commit_snapshot(&runtime, &relation);
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let covered = native.covered_position().unwrap();
    let transitions = native.covered_branch_transitions().unwrap();
    let artifacts = attached_test_medium(&runtime).offered();
    let source = Arc::new(MemberSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                _ => panic!("first capture owns all members"),
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|artifact| (artifact.position(), artifact.as_bytes().to_vec()))
            .collect(),
        damaged: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
        member_reads: std::sync::Mutex::new(Vec::new()),
        commit_reads: std::sync::Mutex::new(Vec::new()),
        commits_damaged: AtomicBool::new(false),
        failed_member: std::sync::Mutex::new(None),
        failures: std::sync::Mutex::new(Default::default()),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            covered,
            transitions,
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut receiving = durable_runtime_with_test_schema();
    bind_durable_medium(&mut receiving, runtime.durability.detach_medium().unwrap());
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let current_members = source.members.keys().copied().collect::<Vec<_>>();
    // Only the relation's owning partition changes. Its endpoints stay cold.
    create_entity_in_partition(
        execution,
        &receiving,
        "unrelated-record",
        PartitionId::main(),
    );
    receiving
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &current_members,
                source: None,
            },
        )
        .unwrap();
    let mut reader = test_owner_begin_transaction_for_main(&receiving);
    assert!(reader
        .read_entity(execution, source_entity)
        .unwrap()
        .effective()
        .is_some());
    let lease = reader
        .basis
        .inner
        .root
        .demand_region_head(execution, source_entity.partition_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        lease
            .head_view()
            .outgoing(source_entity)
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        changed_relations(&relation)
    );
}
