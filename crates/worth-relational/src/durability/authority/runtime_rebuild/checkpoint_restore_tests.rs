use super::*;
use crate::durability::data::RecoveryFailureClass;
use crate::history::data::BranchId;
use crate::identity::data::PartitionId;
use crate::tests::support::*;

#[test]
fn equal_exact_root_reuses_the_readmitted_partition_substrate() {
    let source = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&source, "reuse-root-mirror");
    release_test_commit_snapshot(&source, &committed);
    let checkpoint = source.durability_authority().checkpoint().unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 1);
    assert_eq!(
        checkpoint.branch_roots[0].partition_images,
        checkpoint.partition_images
    );

    let mut restored = persisted_runtime_with_test_schema();
    let roots = restore_branch_root_images(&mut restored, &checkpoint).unwrap();
    let mirror = prepare_partitions(&mut restored, &checkpoint, &roots).unwrap();
    let root = roots.partitions.get(&committed.commit.commit_id).unwrap();
    assert!(shares_first_generation(&mirror, root));
}

#[test]
fn shared_restored_substrate_keeps_a_pinned_reader_exact_after_next_write() {
    let source = persisted_runtime_with_test_schema();
    create_entity(&source, "reuse-reader-seed");
    source.durability_authority().checkpoint().unwrap();
    let plan = source.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();

    let main = BranchId("main".into());
    let pinned = snapshot_for_owner_branch(&recovered, &main);
    create_entity(&recovered, "reuse-reader-next");
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&pinned)
            .unwrap()
            .entities()
            .len(),
        1
    );
    let latest = snapshot_for_owner_branch(&recovered, &main);
    assert_eq!(
        recovered
            .read_truth()
            .read_snapshot(&latest)
            .unwrap()
            .entities()
            .len(),
        2
    );
}

#[test]
fn divergent_sibling_is_not_reused_as_the_storage_mirror() {
    let source = persisted_runtime_with_test_schema();
    create_entity(&source, "reuse-sibling-seed");
    let sibling = create_branch_from_main(&source, "reuse-sibling");
    let sibling_commit =
        create_entity_outcome_on_branch(&source, "reuse-sibling-only", sibling.clone());
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(&source, "reuse-main-only");
    release_test_commit_snapshot(&source, &main_commit);
    let checkpoint = source.durability_authority().checkpoint().unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 2);

    let mut restored = persisted_runtime_with_test_schema();
    let roots = restore_branch_root_images(&mut restored, &checkpoint).unwrap();
    let mirror = prepare_partitions(&mut restored, &checkpoint, &roots).unwrap();
    let main_root = roots.partitions.get(&main_commit.commit.commit_id).unwrap();
    let sibling_root = roots
        .partitions
        .get(&sibling_commit.commit.commit_id)
        .unwrap();
    assert!(shares_first_generation(&mirror, main_root));
    assert!(!shares_first_generation(&mirror, sibling_root));

    let mut plan = source.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.checkpoint = Some(checkpoint);
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    for (branch, expected) in [(BranchId("main".into()), 2), (sibling, 2)] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot)
                .unwrap()
                .entities()
                .len(),
            expected
        );
    }
}

#[test]
fn equal_image_cannot_bypass_global_contract_readmission() {
    let source = persisted_runtime_with_test_schema();
    create_entity(&source, "reuse-contract-corruption");
    let mut checkpoint = source.durability_authority().checkpoint().unwrap();
    assert_eq!(
        checkpoint.branch_roots[0].partition_images,
        checkpoint.partition_images
    );
    checkpoint.aspect_contracts.clear();

    let mut restored = persisted_runtime_with_test_schema();
    let roots = restore_branch_root_images(&mut restored, &checkpoint).unwrap();
    let error = prepare_partitions(&mut restored, &checkpoint, &roots).unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("aspect readmission denied"));
}

fn shares_first_generation(
    left: &partition_images::RestoredPartitions,
    right: &partition_images::RestoredPartitions,
) -> bool {
    let left = left.get(&PartitionId::main()).unwrap();
    let right = right.get(&PartitionId::main()).unwrap();
    std::ptr::eq(
        left.entity_arena.generations.get(0).unwrap(),
        right.entity_arena.generations.get(0).unwrap(),
    )
}
