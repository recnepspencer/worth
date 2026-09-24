use super::*;
use crate::facade::mvcc::WorkerIntentBatch;
use crate::facade::transactions::{BulkEntityCreateIntent, CreateIntent, MutationIntent};
use crate::history::data::BranchId;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::indexes::data::{
    DerivedIndexApplicability, DerivedIndexArtifacts, DerivedIndexEntries, DerivedIndexGeneration,
    DerivedIndexGenerationId, DerivedIndexId, DerivedIndexPublicationStatus,
};
use crate::tests::support::*;

#[test]
fn partition_alias_wire_readmits_legacy_checkpoint_and_omits_envelope_index_cache() {
    let runtime = persisted_runtime_with_test_schema();
    let committed = create_entity_outcome(&runtime, "borrowed-checkpoint-wire");
    release_test_commit_snapshot(&runtime, &committed);
    let mut checkpoint = runtime.durability_authority().checkpoint().unwrap();
    let envelope = checkpoint.envelopes[0].envelope_mut_for_test();
    envelope.derived_index_artifacts = DerivedIndexArtifacts::new(vec![DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(9_001),
        index_id: DerivedIndexId(7_001),
        source_commit_id: committed.commit.commit_id,
        source_branch_id: BranchId("main".into()),
        applicability: DerivedIndexApplicability {
            branch_id: BranchId("main".into()),
            version_id: VersionId(committed.version_id.0),
            schema_version: envelope.schema_version,
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(Default::default()),
    }]);

    let old_wire = rmp_serde::to_vec_named(&PersistedDurableCheckpointFile::from_checkpoint(
        checkpoint.clone(),
    ))
    .unwrap();
    let borrowed_wire =
        rmp_serde::to_vec_named(&PersistedDurableCheckpointFileRef::new(&checkpoint)).unwrap();
    assert!(
        checkpoint.branch_roots[0].partition_images == checkpoint.partition_images,
        "a single main head has the same exact partition image as its storage mirror"
    );
    assert!(borrowed_wire.len() < old_wire.len());
    let root_partition_bytes =
        rmp_serde::to_vec_named(&checkpoint.branch_roots[0].partition_images)
            .unwrap()
            .len();
    assert!(
        old_wire.len() - borrowed_wire.len() >= root_partition_bytes.saturating_sub(256),
        "the wire must omit one complete equal root partition image"
    );
    assert!(
        !checkpoint.envelopes[0]
            .envelope()
            .derived_index_artifacts
            .is_empty(),
        "borrowed encoding does not mutate its input"
    );
    let stored: PersistedDurableCheckpointFile = rmp_serde::from_slice(&borrowed_wire).unwrap();
    assert_eq!(
        stored.checkpoint.partition_alias_format,
        PARTITION_DELTA_FORMAT_VERSION
    );
    assert_eq!(stored.checkpoint.branch_root_partition_aliases_v2.len(), 1);
    assert!(stored.checkpoint.branch_roots[0]
        .partition_images
        .is_empty());
    let restored = stored.readmit().unwrap();
    let legacy = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&old_wire)
        .unwrap()
        .readmit()
        .unwrap();
    assert_eq!(restored.checkpoint, legacy.checkpoint);
    let mut old_alias = PersistedDurableCheckpointFile::from_checkpoint(checkpoint.clone());
    old_alias.checkpoint.partition_alias_format = partition_aliases::PARTITION_ALIAS_FORMAT_VERSION;
    old_alias.checkpoint.branch_root_partition_aliases = vec![checkpoint.branch_roots[0].commit_id];
    old_alias.checkpoint.branch_roots[0]
        .partition_images
        .clear();
    let v1_wire = rmp_serde::to_vec_named(&old_alias).unwrap();
    let v1 = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&v1_wire)
        .unwrap()
        .readmit()
        .unwrap();
    assert_eq!(v1.checkpoint, restored.checkpoint);
    assert!(restored.checkpoint.envelopes[0]
        .envelope()
        .derived_index_artifacts
        .is_empty());
}

#[test]
fn partition_alias_wire_preserves_divergent_sibling_roots_and_pinned_reader() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "partition-alias-seed");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("shared-second-partition").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId(41),
                    kind_id: KindId(1),
                    client_keys: vec![crate::symbols::data::ClientKey::raw("shared-secondary")],
                    field_patches: vec![single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        "shared-secondary",
                    )],
                }),
            )),
        )
        .unwrap();
    let second = transaction.commit(&runtime).unwrap();
    release_test_commit_snapshot(&runtime, &second);
    let main = BranchId("main".into());
    let pinned = snapshot_for_owner_branch(&runtime, &main);
    let sibling = create_branch_from_main(&runtime, "partition-alias-sibling");
    let on_sibling = create_entity_outcome_on_branch(&runtime, "sibling-only", sibling.clone());
    release_test_commit_snapshot(&runtime, &on_sibling);
    let on_main = create_entity_outcome(&runtime, "main-only");
    release_test_commit_snapshot(&runtime, &on_main);

    let checkpoint = runtime.durability_authority().checkpoint().unwrap();
    assert_eq!(checkpoint.branch_roots.len(), 2);
    let wire =
        rmp_serde::to_vec_named(&PersistedDurableCheckpointFileRef::new(&checkpoint)).unwrap();
    let stored = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    assert_eq!(stored.checkpoint.branch_root_partition_aliases_v2.len(), 2);
    let sibling_aliases = stored
        .checkpoint
        .branch_root_partition_aliases_v2
        .iter()
        .find(|root| root.commit_id == on_sibling.commit.commit_id)
        .unwrap();
    assert_eq!(sibling_aliases.shared.len(), 1);
    assert_eq!(sibling_aliases.shared[0].partition_id, PartitionId(41));
    assert!(
        !stored
            .checkpoint
            .branch_roots
            .iter()
            .find(|root| root.commit_id == on_sibling.commit.commit_id)
            .unwrap()
            .partition_images
            .is_empty(),
        "the divergent sibling root remains an inline exact image"
    );
    let restored = stored.readmit().unwrap();
    assert_eq!(restored.checkpoint.branch_roots, checkpoint.branch_roots);
    assert_eq!(
        restored.checkpoint.partition_images,
        checkpoint.partition_images
    );
    assert_eq!(
        runtime
            .read_truth()
            .read_snapshot(&pinned)
            .unwrap()
            .entities()
            .len(),
        2
    );

    let mut plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::NormalRecoveryVerification,
    );
    plan.checkpoint = Some(restored.checkpoint);
    let mut recovered = persisted_runtime_with_test_schema();
    recovered.durability_recovery().recover(plan).unwrap();
    for branch in [main, sibling] {
        let snapshot = snapshot_for_owner_branch(&recovered, &branch);
        assert_eq!(
            recovered
                .read_truth()
                .read_snapshot(&snapshot)
                .unwrap()
                .entities()
                .len(),
            3
        );
    }
}

#[test]
fn partition_alias_wire_rejects_unknown_or_inconsistent_aliases() {
    let runtime = persisted_runtime_with_test_schema();
    create_entity(&runtime, "partition-alias-corruption");
    let checkpoint = runtime.durability_authority().checkpoint().unwrap();
    let wire =
        rmp_serde::to_vec_named(&PersistedDurableCheckpointFileRef::new(&checkpoint)).unwrap();
    let stored = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    assert_eq!(stored.checkpoint.branch_root_partition_aliases_v2.len(), 1);

    let mut unknown = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    unknown.checkpoint.partition_alias_format += 1;
    assert_eq!(
        unknown.readmit().unwrap_err().class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    let mut duplicate = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    duplicate
        .checkpoint
        .branch_root_partition_aliases_v2
        .push(duplicate.checkpoint.branch_root_partition_aliases_v2[0].clone());
    assert_eq!(
        duplicate.readmit().unwrap_err().class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    let mut missing = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    missing.checkpoint.branch_root_partition_aliases_v2[0].commit_id =
        crate::history::data::CommitId(u64::MAX);
    assert_eq!(
        missing.readmit().unwrap_err().class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    let mut inline = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    inline.checkpoint.branch_roots[0].partition_images = checkpoint.partition_images.clone();
    assert_eq!(
        inline.readmit().unwrap_err().class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    let mut corrupt = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    corrupt.checkpoint.partition_images[0]
        .entity_arena
        .generations[0] ^= 1;
    let error = corrupt.readmit().unwrap_err();
    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("shared partition digest mismatch"));

    let mut wrong_digest = rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    wrong_digest.checkpoint.branch_root_partition_aliases_v2[0].shared[0].image_digest[0] ^= 1;
    assert_eq!(
        wrong_digest.readmit().unwrap_err().class,
        RecoveryFailureClass::CorruptCheckpoint
    );

    let mut wrong_position =
        rmp_serde::from_slice::<PersistedDurableCheckpointFile>(&wire).unwrap();
    wrong_position.checkpoint.branch_root_partition_aliases_v2[0].shared[0].position = u32::MAX;
    assert_eq!(
        wrong_position.readmit().unwrap_err().class,
        RecoveryFailureClass::CorruptCheckpoint
    );
}
