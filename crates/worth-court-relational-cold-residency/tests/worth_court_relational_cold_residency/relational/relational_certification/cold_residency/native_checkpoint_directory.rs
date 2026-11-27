//! Actual native checkpoints carry an exact content-addressed member directory.
use super::{
    checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC},
    selected_directory::read_native,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::{
        RecoveryFailureClass, RecoveryVerificationMode as RecoveryMode,
        RelationalDurableMemberLocator, RelationalNativeCheckpoint, RelationalRecoveryStop,
    },
    history::BranchId,
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
    transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent, WorkerIntentBatch},
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
    capture_with_members(execution, runtime, &[])
}

fn capture_with_members(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    current_members: &[RelationalDurableMemberLocator],
) -> RelationalNativeCheckpoint {
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members,
                source: None,
            },
        )
        .unwrap()
}

fn manifest(checkpoint: &RelationalNativeCheckpoint) -> &[u8] {
    field(
        checkpoint.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap(),
        "manifest",
    )
}

fn member_bytes(
    checkpoint: &RelationalNativeCheckpoint,
    selected: RelationalDurableMemberLocator,
) -> Option<&[u8]> {
    checkpoint.members().iter().find_map(|member| match member {
        worth_relational::facade::durability::RelationalCheckpointMember::New {
            locator,
            bytes,
        } if *locator == selected => Some(bytes.bytes()),
        _ => None,
    })
}

fn member_count(
    checkpoint: &RelationalNativeCheckpoint,
    selected: RelationalDurableMemberLocator,
) -> usize {
    checkpoint
        .members()
        .iter()
        .filter(|member| member.locator() == selected)
        .count()
}

fn restore(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalNativeCheckpoint,
) -> RelationalRuntime {
    let mut recovered = memory_runtime();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            native,
            worth_testkit_relational::cold_recovery::members_source(native),
        )
        .unwrap();
    recovered
}

fn expect_durability_failure(
    stop: RelationalRecoveryStop,
) -> worth_relational::facade::durability::DurabilityError {
    let RelationalRecoveryStop::Durability(error) = stop else {
        panic!("expected durability failure: {stop:?}")
    };
    error
}

#[test]
fn member_directory_preserves_divergent_sibling_roots_and_pinned_reader() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity(execution, &runtime, "partition-alias-seed");
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("shared-second-partition").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId(41),
                    kind_id: KindId(1),
                    client_keys: vec!["shared-secondary".into()],
                    field_patches: vec![field_patch("name", "name", "shared-secondary")],
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let second = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &second);
    let main = BranchId("main".into());
    let pinned = snapshot_for_branch(&runtime, &main);
    let sibling = BranchId("partition-alias-sibling".into());
    let (_, fork) = runtime.observe_fork_source(&main).unwrap();
    runtime
        .fork_branch(execution, sibling.clone(), fork)
        .unwrap();
    let basis = runtime
        .admit_branch_basis(&runtime.branch_identity(&sibling).unwrap())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch_create("sibling-only"), execution.allocation_policy())
        .unwrap();
    let on_sibling = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &on_sibling);
    let on_main = create_entity_outcome(execution, &runtime, "main-only");
    release_test_commit_snapshot(&runtime, &on_main);

    let checkpoint = runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let native = capture(execution, &runtime);
    let directory = read_native(&native);
    assert_eq!(directory.roots.len(), 2);
    let sibling_root = directory
        .roots
        .iter()
        .find(|root| root.commit_id == on_sibling.commit.commit_id)
        .unwrap();
    let main_root = directory
        .roots
        .iter()
        .find(|root| root.commit_id == on_main.commit.commit_id)
        .unwrap();
    let sibling_shared = sibling_root
        .regions
        .iter()
        .find(|leaf| leaf.partition_id == PartitionId(41))
        .unwrap();
    let main_shared = main_root
        .regions
        .iter()
        .find(|leaf| leaf.partition_id == PartitionId(41))
        .unwrap();
    assert_eq!(sibling_shared, main_shared);
    assert_ne!(sibling_root.regions[0].head, main_root.regions[0].head);
    let region_keys = directory
        .roots
        .iter()
        .flat_map(|root| &root.regions)
        .chain(&directory.mirror)
        .flat_map(|leaf| [leaf.head, leaf.history])
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        region_keys.len()
            < (sibling_root.regions.len() + main_root.regions.len() + directory.mirror.len()) * 2
    );
    for key in region_keys {
        assert_eq!(
            member_count(&native, key),
            1,
            "equal regions must have exactly one member body across roots and mirror"
        );
    }
    let restored = restore(execution, &native);
    let prior_members: Vec<_> = native.members().iter().map(|m| m.locator()).collect();
    let recaptured = capture_with_members(execution, &restored, &prior_members);
    let restored_directory = read_native(&recaptured);
    for expected in &directory.roots {
        let actual = restored_directory
            .roots
            .iter()
            .find(|root| root.commit_id == expected.commit_id)
            .unwrap();
        assert_eq!(actual.regions, expected.regions);
    }
    assert_eq!(
        restored
            .durability_authority()
            .checkpoint(execution)
            .unwrap()
            .partition_images,
        checkpoint.partition_images
    );
    assert_eq!(
        runtime
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );

    let mut plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    plan.checkpoint = Some(
        restored
            .durability_authority()
            .checkpoint(execution)
            .unwrap(),
    );
    let mut recovered = memory_runtime();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for branch in [main, sibling] {
        let snapshot = snapshot_for_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            3
        );
    }
}

#[test]
fn current_manifest_deduplicates_equal_region_members_and_round_trips() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = memory_runtime();
    create_entity(execution, &runtime, "manifest-roundtrip");
    let native = capture(execution, &runtime);
    let directory = read_native(&native);
    assert_eq!(directory.roots[0].regions, directory.mirror);
    let leaf = &directory.roots[0].regions[0];
    assert!(rows(field(
        native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap(),
        "members"
    ))
    .is_empty());
    assert_eq!(member_count(&native, leaf.head), 1);
    assert_eq!(member_count(&native, leaf.history), 1);
    assert_eq!(directory.commits.len(), 1);
    let reopened = restore(execution, &native);
    assert_eq!(reopened.history().immutable_commit_count(), 1);
    let prior_members: Vec<_> = native.members().iter().map(|m| m.locator()).collect();
    let recaptured = capture_with_members(execution, &reopened, &prior_members);
    let restored_directory = read_native(&recaptured);
    assert_eq!(
        restored_directory.roots[0].regions,
        restored_directory.mirror
    );
    let frame = reopened
        .durability()
        .canonical_commit_artifacts(execution)
        .next()
        .unwrap()
        .unwrap();
    let readmitted =
        worth_relational::facade::durability::RelationalCanonicalCommitArtifact::readmit(
            frame.as_bytes(),
            execution,
        )
        .unwrap();
    assert!(readmitted.envelope().derived_index_artifacts.is_empty());
}

#[test]
fn native_capture_sections_count_current_directory_values() {
    let request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = memory_runtime();
    create_entity(execution, &runtime, "manifest-sections");
    let native = capture(execution, &runtime);
    let sections = native.captured_sections().unwrap();
    assert_eq!(sections.total, native.bytes().len());
    let canonical_bytes: usize = rows(field(manifest(&native), "canonical_members"))
        .iter()
        .map(|entry| {
            let selected =
                rmp_serde::from_slice::<RelationalDurableMemberLocator>(field(entry, "member"))
                    .unwrap();
            member_bytes(&native, selected).unwrap().len()
        })
        .sum();
    assert!(canonical_bytes > 0);
    assert_eq!(sections.envelopes, 0);
    assert_eq!(
        sections.branch_roots,
        field(manifest(&native), "roots").len()
    );
    assert_eq!(
        sections.branch_cells,
        field(manifest(&native), "branch_cells").len()
    );
    assert_eq!(
        sections.partition_mirror,
        field(manifest(&native), "mirror").len()
    );
    assert_eq!(
        sections.derived_indexes,
        field(manifest(&native), "index_definitions").len()
    );
    assert_eq!(
        sections.total,
        sections.envelopes
            + sections.branch_roots
            + sections.branch_cells
            + sections.partition_mirror
            + sections.derived_indexes
            + sections.framing_and_metadata
    );
    let repeated = capture(execution, &runtime);
    assert_eq!(native, repeated);
    assert_eq!(native.clone().captured_sections(), Some(sections));
    assert!(
        RelationalNativeCheckpoint::from_untrusted_bytes(native.bytes())
            .captured_sections()
            .is_none()
    );
}

fn snapshot_for_branch(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> worth_relational::facade::snapshots::SnapshotHandle {
    let basis = runtime
        .admit_branch_basis(&runtime.branch_identity(branch).unwrap())
        .unwrap();
    runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap()
}

mod refusals;

mod kind_authentication;

mod component_headers;
