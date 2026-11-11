use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;
use crate::history::data::BranchId;
use crate::identity::data::PartitionId;
use crate::tests::support::*;

#[test]
fn native_restore_work_counts_revised_history_sibling_roots_and_partition_images() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    let entity = create_entity(execution, &source, "work-seed");
    let small = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    create_entity_in_partition(execution, &source, "work-secondary", PartitionId(41));
    let grown = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        grown.captured_sections().unwrap().partition_mirror
            > small.captured_sections().unwrap().partition_mirror
    );

    let pinned = snapshot_for_owner_branch(&source, &BranchId("main".into()));
    update_entity_and_release_snapshot(execution, &source, entity, "work-revised");
    let sibling = create_branch_from_main(&source, "work-sibling");
    let sibling_commit = create_entity_outcome_on_branch(execution, &source, "work-fork", sibling);
    release_test_commit_snapshot(&source, &sibling_commit);
    let native = source
        .durability_authority()
        .native_checkpoint(
            execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        native.captured_sections().unwrap().envelopes
            > grown.captured_sections().unwrap().envelopes
    );
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        execution,
        native.bytes(),
    )
    .unwrap()
    .checkpoint;
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&source).attachment());
    let outcome = recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &native)
        .unwrap();
    let work = outcome.checkpoint_restore_work.unwrap();
    assert_eq!(work.native_bytes_read, Some(native.bytes().len()));
    assert_eq!(
        work.native_envelopes_readmitted,
        Some(decoded.envelopes.len())
    );
    assert_eq!(work.root_images_verified, decoded.branch_roots.len());
    assert_eq!(
        work.root_partition_images_restored,
        decoded
            .branch_roots
            .iter()
            .map(|root| root.partition_images.len())
            .sum::<usize>()
    );
    assert_eq!(
        work.mirror_partition_images_examined,
        decoded.partition_images.len()
    );
    assert_eq!(
        work.mirror_partitions_reused + work.mirror_partitions_reconstructed,
        decoded.partition_images.len()
    );
    assert!(work.mirror_partitions_reused > 0);
    assert_eq!(work.history_envelopes_routed, decoded.envelopes.len());
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
fn shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    create_entity(execution, &source, "reuse-reader-seed");
    crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &source,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&source).attachment());
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let main = BranchId("main".into());
    let pinned = snapshot_for_owner_branch(&recovered, &main);
    create_entity(execution, &recovered, "reuse-reader-next");
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&pinned, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        1
    );
    let latest = snapshot_for_owner_branch(&recovered, &main);
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&latest, execution)
            .expect("read request remains live")
            .unwrap()
            .entities()
            .len(),
        2
    );
}

#[test]
fn divergent_sibling_is_not_reused_as_the_storage_mirror() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    create_entity(execution, &source, "reuse-sibling-seed");
    let sibling = create_branch_from_main(&source, "reuse-sibling");
    let sibling_commit =
        create_entity_outcome_on_branch(execution, &source, "reuse-sibling-only", sibling.clone());
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(execution, &source, "reuse-main-only");
    release_test_commit_snapshot(&source, &main_commit);
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &source).unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 2);

    let mut restored = durable_runtime_with_test_schema();
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots = restore_branch_root_images(&mut restored, &checkpoint, &mut work, None).unwrap();
    let mirror = prepare_partitions(&mut restored, &checkpoint, &roots, &mut work).unwrap();
    let main_root = roots.partitions.get(&main_commit.commit.commit_id).unwrap();
    let sibling_root = roots
        .partitions
        .get(&sibling_commit.commit.commit_id)
        .unwrap();
    assert!(shares_first_generation(&mirror, main_root));
    assert!(!shares_first_generation(&mirror, sibling_root));

    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &source,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint = Some(checkpoint);
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&source).attachment());
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for (branch, expected) in [(BranchId("main".into()), 2), (sibling, 2)] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot, execution)
                .expect("read request remains live")
                .unwrap()
                .entities()
                .len(),
            expected
        );
    }
}

#[test]
fn recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let source = durable_runtime_with_test_schema();
    let seed = create_entity_outcome(execution, &source, "catalog-relink-seed");
    release_test_commit_snapshot(&source, &seed);
    let sibling = create_branch_from_main(&source, "catalog-relink-sibling");
    let sibling_commit =
        create_entity_outcome_on_branch(execution, &source, "catalog-relink-fork", sibling.clone());
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(execution, &source, "catalog-relink-main");
    release_test_commit_snapshot(&source, &main_commit);
    crate::tests::support::checkpoint_durably(execution, &source).unwrap();

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &source,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    bind_durable_medium(&mut recovered, attached_test_medium(&source).attachment());
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    for commit_id in [
        seed.commit.commit_id,
        sibling_commit.commit.commit_id,
        main_commit.commit.commit_id,
    ] {
        let original = source.history.commit_artifact(commit_id).unwrap();
        let readmitted = recovered.history.commit_artifact(commit_id).unwrap();
        assert_eq!(readmitted.identity(), original.identity());
        assert_eq!(readmitted.parentage(), original.parentage());
        assert_eq!(readmitted.roots(), original.roots());
        assert_eq!(
            readmitted
                .canonical_payload_digest()
                .expect("resident fixture has a canonical digest"),
            original
                .canonical_payload_digest()
                .expect("resident fixture has a canonical digest")
        );
    }
}

fn shares_first_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
) -> bool {
    shares_partition_generation(left, right, PartitionId::main())
}

fn shares_partition_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
    partition_id: PartitionId,
) -> bool {
    let left = left.get(&partition_id).unwrap();
    let right = right.get(&partition_id).unwrap();
    std::ptr::eq(
        left.entity_arena.generations.get(0).unwrap(),
        right.entity_arena.generations.get(0).unwrap(),
    )
}
