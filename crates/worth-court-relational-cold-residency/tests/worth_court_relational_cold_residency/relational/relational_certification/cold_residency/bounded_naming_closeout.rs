//! Closeout courts for Store M4.3f:
//! - Damaged naming/group pages fail closed as typed `StorageUnavailable` /
//!   `ColdStorageUnavailable` across observation reads, commits, and tail replay.
//! - `advance_naming_retention_floor` preserves exact answers at `>= floor`
//!   while refusing `< floor` with `BasisPruned`.
use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalDurableCheckpointSource,
    RelationalDurableRecoverySource, RelationalRecoveryStop,
};
use worth_relational::facade::history::CommitId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, PersistentNameIndexDenial, PersistentNameKey, PersistentNameResolutionDenial,
    PersistentNameSelection, RelationalPersistentNameReadDenial,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_relational::facade::transactions::{
    CommitPreparationReason, EntityMutationIntent, MutationIntent, TransactionCommitError,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, delete_entity, durable_named_runtime, group_scheme, naming_pages,
    observe_branch, publish_group_partition, reopen_named, GROUP_SCHEME,
};

#[test]
fn damaged_naming_page_fails_closed_on_reads_commits_and_tail_replay() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();
    let (_, entities, names) =
        create_named_entities(request, &producer, &main, &["dmg-1".into(), "dmg-2".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        entities[0],
        "dmg-group",
        vec![vec![
            (names[0], AggregateMemberRole::Anchor),
            (names[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let (c1, _) = capture_external_checkpoint(request, &producer);
    let source = immutable_source(&c1, &medium);
    let covered_pos = c1.covered_position().unwrap();
    let covered_trans = c1.covered_branch_transitions().unwrap();
    let pages = naming_pages(&source);
    assert!(!pages.is_empty());

    // Publish a tail commit that deletes dmg-1 (touches naming basis during tail replay).
    delete_entity(request, &producer, &main, entities[0]);

    // 1. Read and live commit fail closed when naming pages are damaged.
    let reopened = reopen_named(request, c1.clone(), source.clone(), &medium, u64::MAX / 4);
    for locator in &pages {
        source
            .failures
            .lock()
            .unwrap()
            .insert(ColdRead::Member(*locator), Cause::Damaged);
    }
    let head = observe_branch(&reopened, &main);
    let read_err = reopened
        .with_persistent_names_at_observation(request, &head.observation(), |view| {
            view.resolve(
                PersistentNameKey::Entity(names[0]),
                &PersistentNameSelection::Exact,
            )
        })
        .unwrap_err();
    assert!(
        matches!(
            read_err,
            RelationalPersistentNameReadDenial::IndexUnavailable(
                PersistentNameIndexDenial::StorageUnavailable(Cause::Damaged)
            )
        ),
        "expected StorageUnavailable(Damaged), got {read_err:?}"
    );

    let mut tx = reopened
        .begin_branch_transaction(&head, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("touch-damaged").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entities[0],
                fields: field_patch("name", "name", "updated"),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let mut plan = CommitNamingPlan::default();
    plan.declare_group_partition(
        group_scheme(),
        Vec::new(),
        vec![vec![(names[0], AggregateMemberRole::Anchor)]],
    );
    tx.attach_naming_plan(plan);
    let commit_err = tx.commit(request, &reopened).unwrap_err();
    assert!(
        matches!(
            commit_err,
            TransactionCommitError::Preparation { ref error, .. }
                if matches!(error.reason(), CommitPreparationReason::StorageUnavailable(_))
        ),
        "commit over damaged naming page must return typed StorageUnavailable, got {commit_err:?}"
    );

    // 2. Tail replay over damaged naming page returns typed ColdStorageUnavailable.
    let tail_commits: Vec<_> = medium
        .committed_artifacts()
        .unwrap()
        .into_iter()
        .filter(|artifact| artifact.position() > covered_pos)
        .collect();
    let mut replayed = durable_named_runtime(&medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            covered_pos,
            covered_trans,
            c1,
        )),
        tail_commits,
        Vec::new(),
        source,
    );
    let plan = replayed
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let replay_err = replayed
        .durability_recovery()
        .recover(request, plan)
        .unwrap_err();
    assert!(
        matches!(
            replay_err,
            RelationalRecoveryStop::Durability(ref d)
                if d.class == RecoveryFailureClass::ColdStorageUnavailable
        ),
        "tail replay over damaged naming page must return ColdStorageUnavailable, got {replay_err:?}"
    );
}

#[test]
fn advance_naming_retention_floor_preserves_ge_floor_and_refuses_lt_floor() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_named_runtime(&medium, u64::MAX / 4);
    let main = runtime.main_branch_identity().branch_id().clone();

    let (c1, e1, n1) =
        create_named_entities(request, &runtime, &main, &["ret-a".into(), "ret-b".into()]);
    let obs_c1 = observe_branch(&runtime, &main);
    let _c2 = publish_group_partition(
        request,
        &runtime,
        &main,
        e1[0],
        "g-step-1",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let _c3 = delete_entity(request, &runtime, &main, e1[1]);
    let (c4, e4, n4) = create_named_entities(request, &runtime, &main, &["ret-c".into()]);
    let _c5 = publish_group_partition(
        request,
        &runtime,
        &main,
        e4[0],
        "g-step-2",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n4[0], AggregateMemberRole::Ordinary),
        ]],
    );
    let obs_head = observe_branch(&runtime, &main);
    let floor = c4.commit.commit_id;
    assert!(c1.commit.commit_id < floor);

    let keys = [
        PersistentNameKey::Entity(n1[0]),
        PersistentNameKey::Entity(n1[1]),
        PersistentNameKey::Entity(n4[0]),
    ];
    let oracle = runtime
        .with_persistent_names_at_observation(request, &obs_head.observation(), |view| {
            let group = view.group_of_member(GROUP_SCHEME, &n1[0]);
            let resolutions: Vec<_> = keys
                .iter()
                .map(|k| view.resolve(*k, &PersistentNameSelection::Exact).result)
                .collect();
            (group, resolutions)
        })
        .unwrap();

    runtime
        .advance_naming_retention_floor(&main, floor)
        .unwrap();
    // Lower floor is a monotonic no-op.
    runtime
        .advance_naming_retention_floor(&main, CommitId(1))
        .unwrap();

    // Reads strictly below `floor` refuse with BasisPruned.
    runtime
        .with_persistent_names_at_observation(request, &obs_c1.observation(), |view| {
            let err = view
                .resolve(keys[0], &PersistentNameSelection::Exact)
                .result
                .unwrap_err();
            assert!(matches!(
                err,
                PersistentNameResolutionDenial::BasisPruned { .. }
            ));
        })
        .unwrap();

    // Reads at `>= floor` agree identically with oracle.
    let actual = runtime
        .with_persistent_names_at_observation(request, &obs_head.observation(), |view| {
            let group = view.group_of_member(GROUP_SCHEME, &n1[0]);
            let resolutions: Vec<_> = keys
                .iter()
                .map(|k| view.resolve(*k, &PersistentNameSelection::Exact).result)
                .collect();
            (group, resolutions)
        })
        .unwrap();
    assert_eq!(actual, oracle);
}
