//! Genuine two-object post-NoRelease history folds into a selected tag-7
//! Batch/Accumulator checkpoint, then survives a fresh C.8/Store reopen.

use std::fs;

use {
    worth_proof::TransitionOutcome, worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving,
    worth_store_testkit_recovery_runtime::production_entry::pending_wal_world,
    worth_store_testkit_recovery_runtime::production_entry::release_reopen,
};
use {
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::RecordArtifactFile,
};
use {
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

#[test]
fn two_killed_v3_batches_fold_at_next_checkpoint_and_reopen() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "two-object C8 blocked: kind={:?}, page={:?}, effects={}",
                blocked.cause(),
                blocked.evidence().planning_denial,
                blocked.recovery_effects(),
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "two-object Store handoff: handoff={:?}, reopen={:?}, effects={}",
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure(),
                indeterminate.recovery_effects(),
            ),
            other => panic!("two-object C8 outcome: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("ordered two-batch Store custody seal");
    let serving = worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x9c; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("ordered released checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();

    let bytes = std::fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        worth_store_testkit_recovery_runtime::production_entry::release_reopen::selected_release_certificates_from_bytes(&bytes);
    assert_eq!(
        batches.len(),
        2,
        "both genuine V3 WAL batches must be folded"
    );
    assert_ne!(
        batches[0].descriptor_record(),
        batches[1].descriptor_record()
    );
    assert!(batches[0].candidate_root_generation() < batches[1].candidate_root_generation());
    assert_eq!(
        accumulator.base().tip(),
        batches[1].tip_provenance().unwrap()
    );
    assert_eq!(
        accumulator.base().cumulative_dropped(),
        batches[1].cumulative_dropped()
    );
    assert_eq!(
        accumulator.base().cumulative_digest(),
        batches[1].cumulative_digest()
    );
    let heads =
        worth_store_testkit_recovery_runtime::release_reopen::selected_head_oracle::selected_heads(
            world.root(),
            accumulator,
        );
    assert_eq!(heads.len(), 2);
    assert_ne!(heads[0].key(), heads[1].key());

    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "fresh folded C8 blocked: kind={:?}, artifact={:?}, planning={:?}, effects={}",
                blocked.cause(),
                blocked.evidence().artifact,
                blocked.evidence().planning_denial,
                blocked.recovery_effects(),
            ),
            worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
                indeterminate,
            ) => panic!(
                "fresh folded Store handoff: handoff={:?}, reopen={:?}, effects={}",
                indeterminate.handoff_failure(),
                indeterminate.reopen_failure(),
                indeterminate.recovery_effects(),
            ),
            other => panic!("fresh C8/Store must admit folded checkpoint: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("selected Batch/Accumulator custody after fresh reopen");
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn post_seal_intermediate_history_root_substitution_denies_serving() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root()));
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("two-object C8 recovery must construct a seal before mutation: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("ordered two-batch Store custody seal");
    // Change a valid, intermediate historical root header without touching
    // its route/free membership or the current selected root. A raw bit flip
    // would fail framing before the recovered-custody fingerprint is tested.
    let records = world.root().join("families/records");
    let selected_path = records
        .join(worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name());
    let selected_before = std::fs::read(&selected_path).unwrap();
    let path = records.join("roots").join(
        worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 12 }
            .file_name(),
    );
    let (before, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(
        &std::fs::read(&path).unwrap(),
        u16::MAX,
    )
    .unwrap();
    let changed = worth_store_physical_format::DurablePhysicalRootManifest::builder(
        before.generation(),
        before.tree_identity(),
        before.node_capacity(),
        before.free_space_checksum(),
    )
    .record_count(before.record_count())
    .next_block(before.next_block().checked_add(1).unwrap())
    .next_segment_block(before.next_segment_block())
    .routing_root(before.routing_root())
    .segment_root(before.segment_root())
    .free_space_root(before.free_space_root())
    .latest_blob_publication(before.latest_blob_publication())
    .latest_blob_quarantine(before.latest_blob_quarantine())
    .tier_epoch_anchor(before.tier_epoch_anchor())
    .derived_family_directory(before.derived_family_directory())
    .last_inline_record(before.last_inline_record())
    .last_inline_segment(before.last_inline_segment())
    .admit()
    .expect("valid alternate historical root header");
    assert_ne!(changed, before);
    std::fs::write(&path, changed.encode(format)).unwrap();
    let (decoded, decoded_format) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(
            &std::fs::read(&path).unwrap(),
            u16::MAX,
        )
        .unwrap();
    assert_eq!((decoded, decoded_format), (changed, format));
    assert_eq!(std::fs::read(&selected_path).unwrap(), selected_before);
    worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::open_serving_with_seal_expect_mismatch(world.root(), seal);
}
#[cfg(all())]
#[path = "directory_media/mod.rs"]
pub(crate) mod directory_media;
#[cfg(all())]
#[path = "memory_budget/mod.rs"]
pub(crate) mod memory_budget;
