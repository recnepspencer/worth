//! A pending V3 batch that names a per-object predecessor, killed at its
//! durable descriptor WAL. Serving admitted every earlier release without a
//! new checkpoint, so recovery must admit the pending batch from the same
//! checkpoint and ordered history, and then rejoin it as completed history.

pub use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, crate::production_entry::certified_release_serving, crate::production_entry::pending_wal_world, crate::production_entry::release_reopen};
pub use {crate::production_entry::pending_wal_world::PendingWalWorld};
pub use {std::fs};
pub use {worth_store_recovery_runtime::PhysicalRecoveryLimitDimension, worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::RecoveredPhysicalRuntimeHandoff, worth_store_recovery_runtime::WorthStoreRecovery};

/// What the first reopen of the frontier world needs. Fixed: about 140 root
/// steps stand above its checkpoint, and each is charged what its member
/// declares it wrote, whichever blocks the ingest landed in.
/// Every root read costs one entry, found or not. Among them: the successor
/// root the candidate probe finds absent (1), and the historical roots a
/// blob record is looked up under (2).
pub const FRONTIER_NEED: u64 = 1003;

pub fn recover(world: &PendingWalWorld, stage: &str) -> RecoveredPhysicalRuntimeHandoff {
    match worth_store_recovery_runtime::WorthStoreRecovery::recover(world.recovery_request()) {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => *handoff,
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(block) => panic!(
            "{stage} blocked: kind={:?}; artifact={:?}; limit={:?}; sources={:?}; cause={:?}; \
             effects={}",
            block.cause(),
            block.evidence().artifact.as_deref(),
            block.cause().limit(),
            block.evidence().source_denials,
            block.evidence().planning_denial,
            block.recovery_effects()
        ),
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
            "{stage} indeterminate: handoff={:?}; reopen={:?}; effects={}",
            failure.handoff_failure(),
            failure.reopen_failure(),
            failure.recovery_effects()
        ),
        _ => panic!("{stage} did not recover"),
    }
}

/// One manifest entry short of `need` is that limit, reached by the refused
/// charge, and `need` recovers. Neither attempt is sealed, so the world is
/// left for the recovery that is.
pub fn assert_needs_exactly(world: &PendingWalWorld, need: u64, stage: &str) {
    let recover = |entries| {
        worth_store_recovery_runtime::WorthStoreRecovery::recover(
            crate::production_entry::certified_release_serving::recovery_request::request_for_long_ingest_with_manifest_entries(
                world.root(),
                entries,
            ),
        )
    };
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = recover(need - 1) else {
        panic!("{stage}: {} entries must block", need - 1)
    };
    assert_eq!(blocked.recovery_effects(), 0);
    assert_eq!(
        blocked.cause().limit().map(|limit| (
            limit.dimension(),
            limit.observed(),
            limit.admitted()
        )),
        Some((
            worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestEntries,
            need,
            need - 1
        )),
        "{stage}: denial={:?}",
        blocked.evidence().planning_denial,
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = recover(need) else {
        panic!("{stage}: {need} entries must recover")
    };
    drop(handoff);
}

/// Recovers the parked batch, seals Serving on it, and publishes a
/// head-bearing checkpoint whose source roster carries every released object.
pub fn recover_and_checkpoint(world: &PendingWalWorld, stage: &str, key: [u8; 32]) -> usize {
    let heads = checkpoint_heads(world, stage, key);
    assert!(heads.iter().all(|head| !head.terminal()));
    heads.len()
}

/// The heads of the checkpoint published over the recovered world.
pub fn checkpoint_heads(
    world: &PendingWalWorld,
    stage: &str,
    key: [u8; 32],
) -> Vec<worth_store_physical_format::ReleaseCustodyHeadEntryV1> {
    let root = world.root();
    let seal = recover(world, stage)
        .into_core()
        .certification_release_media_into_checkpoint_custody();
    let serving = crate::production_entry::certified_release_serving::admit_serving_with_seal(
        root,
        seal.expect("recovered batch custody seal"),
    );
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new(key),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("{stage}: head-bearing checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    serving.close();
    let checkpoint = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let (_, accumulator) = crate::production_entry::release_reopen::selected_release_certificates_from_bytes(&checkpoint);
    crate::release_reopen::selected_head_oracle::selected_heads(root, accumulator)
}

/// The pending batch must redo and seal, and the next recovery must rejoin
/// it as completed history, both from the unchanged checkpoint.
pub fn assert_pending_then_completed(world: &PendingWalWorld, stage: &str) {
    let root = world.root();
    let checkpoint = std::fs::read(root.join("families/checkpoint.current")).unwrap();
    let pending = recover(world, stage);
    assert!(pending.core().recovery_effect_count() > 0);
    let seal = pending
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("Store must seal the pending successor");
    crate::production_entry::certified_release_serving::admit_serving_with_seal(root, seal).close();
    assert_eq!(
        std::fs::read(root.join("families/checkpoint.current")).unwrap(),
        checkpoint,
        "{stage}: the redone successor must stay above the same checkpoint",
    );
    let completed = recover(world, stage);
    assert_eq!(completed.core().recovery_effect_count(), 0);
    let seal = completed
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("Store must seal the completed successor");
    crate::production_entry::certified_release_serving::open_serving_with_seal_without_checkpoint(root, seal);
}

pub fn ordered_release_above_a_head_checkpoint() -> PendingWalWorld {
    let world = pending_wal_world::first();
    world.kill_second_after_certified_retirement();
    world.kill_distinct_release_before_checkpoint();
    world.kill_successor_of_first_object();
    world
}

pub fn successor_of_a_checkpoint_head_above_foreign_history() -> PendingWalWorld {
    let world = pending_wal_world::first();
    assert_eq!(
        recover_and_checkpoint(&world, "first object", [0xd1; 32]),
        1
    );
    world.kill_distinct_release_before_checkpoint();
    assert_eq!(
        recover_and_checkpoint(&world, "distinct object", [0xd2; 32]),
        2
    );
    world.kill_successor_of_first_object();
    world.kill_successor_of_distinct_object();
    world
}

/// A checkpoint heads the first object, and the batch that would complete
/// its release is killed above that checkpoint.
pub fn terminal_successor_of_a_checkpoint_head() -> PendingWalWorld {
    let world = pending_wal_world::first();
    assert_eq!(
        recover_and_checkpoint(&world, "first object", [0xd3; 32]),
        1
    );
    world.kill_full_batch_successor_of_first_object();
    world
}