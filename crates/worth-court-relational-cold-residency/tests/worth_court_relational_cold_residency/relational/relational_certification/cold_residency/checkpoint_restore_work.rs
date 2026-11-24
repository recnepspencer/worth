//! Work and catalog parity observed through native capture and actual restart.
use super::selected_directory::read_native;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RelationalNativeCheckpoint,
    history::BranchId,
    identity::{EntityId, KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
    transactions::{
        CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_publication::{batch_create, create_entity_outcome, field_patch, memory_runtime},
    ordinary_records::create_entity,
    shared_relational_certification_test_execution_policy::serial_request,
};

fn capture(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> RelationalNativeCheckpoint {
    runtime
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}

fn fork(execution: &ExecutionRequest<'_, '_>, source: &RelationalRuntime, name: &str) -> BranchId {
    let (_, from) = source
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    let sibling = BranchId(name.into());
    source
        .fork_branch(execution, sibling.clone(), from)
        .unwrap();
    sibling
}

fn commit_sibling(
    execution: &ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
    branch: &BranchId,
    key: &str,
) -> worth_relational::facade::transactions::CommitResult {
    let basis = source
        .admit_branch_basis(&source.branch_identity(branch).unwrap())
        .unwrap();
    let mut transaction = source
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch_create(key), execution.allocation_policy())
        .unwrap();
    transaction.commit(execution, source).unwrap()
}

fn create_in_partition(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition_id: PartitionId,
) {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id,
                    kind_id: KindId(1),
                    client_key: key.into(),
                    fields: field_patch("name", "name", key),
                },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}

fn update(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    value: &str,
) {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: field_patch("name", "name", value),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}

#[test]
fn native_restore_work_counts_revised_history_sibling_roots_and_partition_images() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = memory_runtime();
    let entity = create_entity(execution, &source, "work-seed");
    let small = capture(execution, &source);
    create_in_partition(execution, &source, "work-secondary", PartitionId(41));
    let grown = capture(execution, &source);
    assert!(
        grown.captured_sections().unwrap().partition_mirror
            > small.captured_sections().unwrap().partition_mirror
    );

    let main_basis = source
        .admit_branch_basis(&source.main_branch_identity())
        .unwrap();
    let pinned = source
        .snapshots()
        .snapshot_for_observation(&main_basis.observation())
        .unwrap();
    update(execution, &source, entity, "work-revised");
    let sibling = fork(execution, &source, "work-sibling");
    let sibling_commit = commit_sibling(execution, &source, &sibling, "work-fork");
    release_test_commit_snapshot(&source, &sibling_commit);
    let native = capture(execution, &source);
    assert!(
        native.captured_sections().unwrap().envelopes
            > grown.captured_sections().unwrap().envelopes
    );
    let decoded = read_native(&native);
    let mut recovered = memory_runtime();
    let outcome = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &native)
        .unwrap();
    let work = outcome.checkpoint_restore_work.unwrap();
    assert_eq!(work.native_bytes_read, Some(native.bytes().len()));
    assert_eq!(
        work.native_envelopes_readmitted,
        Some(decoded.commits.len())
    );
    assert_eq!(work.root_images_verified, decoded.roots.len());
    assert_eq!(
        work.root_partition_images_restored,
        decoded
            .roots
            .iter()
            .map(|root| root.regions.len())
            .sum::<usize>()
    );
    assert_eq!(work.mirror_partition_images_examined, decoded.mirror.len());
    assert_eq!(
        work.mirror_partitions_reused + work.mirror_partitions_reconstructed,
        decoded.mirror.len()
    );
    assert!(work.mirror_partitions_reused > 0);
    assert_eq!(work.history_envelopes_routed, decoded.commits.len());
    assert_eq!(work.branch_cells_readmitted, decoded.branch_cells.len());
    assert_eq!(
        work.index_definitions_readmitted,
        decoded.index_definitions.len()
    );
    assert_eq!(
        source
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );
}

#[test]
fn recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = memory_runtime();
    let seed = create_entity_outcome(execution, &source, "catalog-relink-seed");
    release_test_commit_snapshot(&source, &seed);
    let sibling = fork(execution, &source, "catalog-relink-sibling");
    let sibling_commit = commit_sibling(execution, &source, &sibling, "catalog-relink-fork");
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(execution, &source, "catalog-relink-main");
    release_test_commit_snapshot(&source, &main_commit);
    source.durability_authority().checkpoint(execution).unwrap();

    let native = capture(execution, &source);
    let original = read_native(&native);
    let mut recovered = memory_runtime();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &native)
        .unwrap();
    let readmitted = read_native(&capture(execution, &recovered));
    for commit_id in [
        seed.commit.commit_id,
        sibling_commit.commit.commit_id,
        main_commit.commit.commit_id,
    ] {
        assert_eq!(
            recovered
                .history()
                .immutable_commit_identity(commit_id)
                .unwrap(),
            source
                .history()
                .immutable_commit_identity(commit_id)
                .unwrap()
        );
        let select = |directory: &super::selected_directory::Directory| {
            let entry = directory
                .commits
                .iter()
                .find(|entry| entry.commit.commit_id == commit_id)
                .unwrap();
            (entry.commit.parents.clone(), entry.root.clone())
        };
        let (readmitted_parents, readmitted_roots) = select(&readmitted);
        let (original_parents, original_roots) = select(&original);
        assert_eq!(readmitted_parents, original_parents);
        assert_eq!(readmitted_roots, original_roots);
        let digest = |runtime: &RelationalRuntime| {
            runtime
                .durability()
                .canonical_commit_artifacts(execution)
                .map(Result::unwrap)
                .find(|artifact| artifact.commit_id() == commit_id)
                .expect("resident fixture has a canonical digest")
                .payload_digest()
        };
        assert_eq!(digest(&recovered), digest(&source));
    }
}
