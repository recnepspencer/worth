//! Genuine two-object post-NoRelease history folds into a selected tag-7
//! Batch/Accumulator checkpoint, then survives a fresh C.8/Store reopen.

use std::fs;

use super::*;
use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[path = "pending_wal_fold/directory_media.rs"]
mod directory_media;
#[path = "pending_wal_fold/memory_budget.rs"]
mod memory_budget;

#[test]
fn two_killed_v3_batches_fold_at_next_checkpoint_and_reopen() {
    let world = super::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome =
        WorthStoreRecovery::recover(super::certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "two-object C8 blocked: kind={:?}, page={:?}, effects={}",
                blocked.kind,
                blocked.evidence().planning_denial,
                blocked.recovery_effects(),
            ),
            PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) => panic!(
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
        .into_checkpoint_custody()
        .expect("ordered two-batch Store custody seal");
    let serving = super::certified_release_serving::admit_serving_with_seal(world.root(), seal);
    let checkpoint = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x9c; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(checkpoint).into_raw()
    else {
        panic!("ordered released checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();

    let bytes = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    let (batches, accumulator) =
        super::release_reopen::selected_release_certificates_from_bytes(&bytes);
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
        super::release_reopen::selected_head_oracle::selected_heads(world.root(), accumulator);
    assert_eq!(heads.len(), 2);
    assert_ne!(heads[0].key(), heads[1].key());

    let outcome =
        WorthStoreRecovery::recover(super::certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            PhysicalRecoveryOutcome::Blocked(blocked) => panic!(
                "fresh folded C8 blocked: kind={:?}, artifact={:?}, planning={:?}, effects={}",
                blocked.kind,
                blocked.evidence().artifact,
                blocked.evidence().planning_denial,
                blocked.recovery_effects(),
            ),
            PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) => panic!(
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
        .into_checkpoint_custody()
        .expect("selected Batch/Accumulator custody after fresh reopen");
    super::certified_release_serving::open_serving_with_seal_without_checkpoint(world.root(), seal);
}

#[test]
fn post_seal_intermediate_history_root_substitution_denies_serving() {
    let world = super::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let outcome =
        WorthStoreRecovery::recover(super::certified_release_serving::request(world.root()));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("two-object C8 recovery must construct a seal before mutation: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("ordered two-batch Store custody seal");
    // Change a valid, intermediate historical root header without touching
    // its route/free membership or the current selected root. A raw bit flip
    // would fail framing before the recovered-custody fingerprint is tested.
    let records = world.root().join("families/records");
    let selected_path = records.join(RecordArtifactFile::CurrentRootSelector.file_name());
    let selected_before = fs::read(&selected_path).unwrap();
    let path = records
        .join("roots")
        .join(RecordArtifactFile::RootManifest { generation: 12 }.file_name());
    let (before, format) =
        DurablePhysicalRootManifest::decode(&fs::read(&path).unwrap(), u16::MAX).unwrap();
    let changed = DurablePhysicalRootManifest::builder(
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
    fs::write(&path, changed.encode(format)).unwrap();
    let (decoded, decoded_format) =
        DurablePhysicalRootManifest::decode(&fs::read(&path).unwrap(), u16::MAX).unwrap();
    assert_eq!((decoded, decoded_format), (changed, format));
    assert_eq!(fs::read(&selected_path).unwrap(), selected_before);
    super::certified_release_serving::open_serving_with_seal_expect_mismatch(world.root(), seal);
}
