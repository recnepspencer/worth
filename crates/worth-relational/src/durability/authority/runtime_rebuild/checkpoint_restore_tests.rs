use super::*;
use crate::durability::data::RecoveryFailureClass;
use crate::facade::mvcc::WorkerIntentBatch;
use crate::facade::transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent};
use crate::history::data::BranchId;
use crate::identity::data::KindId;
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
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots = restore_branch_root_images(&mut restored, &checkpoint, &mut work).unwrap();
    let mirror = prepare_partitions(&mut restored, &checkpoint, &roots, &mut work).unwrap();
    let root = roots.partitions.get(&committed.commit.commit_id).unwrap();
    assert!(shares_first_generation(&mirror, root));
}

#[test]
fn native_restore_work_counts_revised_history_sibling_roots_and_partition_images() {
    let source = persisted_runtime_with_test_schema();
    let entity = create_entity(&source, "work-seed");
    let small = source.durability_authority().native_checkpoint().unwrap();
    create_entity_in_partition(&source, "work-secondary", PartitionId(41));
    let grown = source.durability_authority().native_checkpoint().unwrap();
    assert!(
        grown.captured_sections().unwrap().partition_mirror
            > small.captured_sections().unwrap().partition_mirror
    );

    let pinned = snapshot_for_owner_branch(&source, &BranchId("main".into()));
    update_entity_and_release_snapshot(&source, entity, "work-revised");
    let sibling = create_branch_from_main(&source, "work-sibling");
    let sibling_commit = create_entity_outcome_on_branch(&source, "work-fork", sibling);
    release_test_commit_snapshot(&source, &sibling_commit);
    let native = source.durability_authority().native_checkpoint().unwrap();
    assert!(
        native.captured_sections().unwrap().envelopes
            > grown.captured_sections().unwrap().envelopes
    );
    let decoded = crate::durability::log::native_file_codec::decode_checkpoint(native.bytes())
        .unwrap()
        .checkpoint;
    let mut recovered = persisted_runtime_with_test_schema();
    let outcome = recovered
        .durability_recovery()
        .restore_native_checkpoint(&native)
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
            .read_snapshot(&pinned)
            .unwrap()
            .entities()
            .len(),
        2
    );
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
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots = restore_branch_root_images(&mut restored, &checkpoint, &mut work).unwrap();
    let mirror = prepare_partitions(&mut restored, &checkpoint, &roots, &mut work).unwrap();
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
fn divergent_mirror_partition_rebuilds_cross_partition_adjacency_after_reuse() {
    let source = persisted_runtime_with_test_schema();
    let main_entity = create_entity(&source, "partial-mirror-main");
    let mut transaction = test_owner_begin_transaction_for_main(&source);
    transaction
        .push_batch(WorkerIntentBatch::new("partial-mirror-secondary").push(
            MutationIntent::Create(CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id: PartitionId(41),
                kind_id: KindId(1),
                client_keys: vec![crate::symbols::data::ClientKey::raw("secondary")],
                field_patches: vec![single_string_aspect_field_patch(
                    aspect_key("name"),
                    field_key("name"),
                    "secondary",
                )],
            })),
        ))
        .unwrap();
    let committed = transaction.commit(&source).unwrap();
    let secondary = changed_entities(&committed)[0];
    release_test_commit_snapshot(&source, &committed);
    create_relation_in_partition(
        &source,
        main_entity,
        secondary,
        "cross-partition",
        PartitionId(31),
    );
    let mut checkpoint = source.durability_authority().checkpoint().unwrap();
    let main_image = checkpoint
        .partition_images
        .iter_mut()
        .find(|image| image.partition_id == PartitionId::main())
        .unwrap();
    main_image.entity_arena.snapshot_pins[0] += 1;

    let mut restored = persisted_runtime_with_test_schema();
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots = restore_branch_root_images(&mut restored, &checkpoint, &mut work).unwrap();
    let mirror = prepare_partitions(&mut restored, &checkpoint, &roots, &mut work).unwrap();
    let root_commit = checkpoint.branch_roots[0].commit_id;
    let root = roots.partitions.get(&root_commit).unwrap();
    assert!(!shares_partition_generation(
        &mirror,
        root,
        PartitionId::main()
    ));
    assert!(shares_partition_generation(&mirror, root, PartitionId(41)));
    let mirror_relation = mirror.get(&PartitionId(31)).unwrap();
    let root_relation = root.get(&PartitionId(31)).unwrap();
    assert!(std::ptr::eq(
        mirror_relation.relation_arena.generations.get(0).unwrap(),
        root_relation.relation_arena.generations.get(0).unwrap(),
    ));
}

#[test]
fn recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads() {
    let source = persisted_runtime_with_test_schema();
    let seed = create_entity_outcome(&source, "catalog-relink-seed");
    release_test_commit_snapshot(&source, &seed);
    let sibling = create_branch_from_main(&source, "catalog-relink-sibling");
    let sibling_commit =
        create_entity_outcome_on_branch(&source, "catalog-relink-fork", sibling.clone());
    release_test_commit_snapshot(&source, &sibling_commit);
    let main_commit = create_entity_outcome(&source, "catalog-relink-main");
    release_test_commit_snapshot(&source, &main_commit);
    source.durability_authority().checkpoint().unwrap();

    let plan = source.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
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
            readmitted.canonical_payload_digest(),
            original.canonical_payload_digest()
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
    let mut work = crate::durability::data::CheckpointRestoreWork::default();
    let roots = restore_branch_root_images(&mut restored, &checkpoint, &mut work).unwrap();
    let error = prepare_partitions(&mut restored, &checkpoint, &roots, &mut work).unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("aspect readmission denied"));
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
