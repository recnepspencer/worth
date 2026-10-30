//! Ordering the retirements above the checkpoint charges what every root
//! step among them declares it wrote. A request that admits one manifest
//! entry too few must be told so, with the count the refused charge reached,
//! and the same request with that one entry more recovers.

pub use crate::production_entry::pending_wal_world::PendingWalWorld;
pub use worth_store::physical_runtime::ServingPhysicalRuntime;
pub use {
    crate::production_entry::certified_release_serving,
    crate::production_entry::pending_successor_above_history,
    crate::production_entry::pending_wal_world,
    crate::production_entry::published_above_checkpoint,
};
pub use {
    worth_store_recovery_runtime::PhysicalRecoveryLimitDimension,
    worth_store_recovery_runtime::PhysicalRecoveryOutcome,
    worth_store_recovery_runtime::WorthStoreRecovery,
};

/// What the idle world needs: three two-chunk objects published above the
/// checkpoint, no release. Fixed, not measured: the charge counts what the
/// workload wrote and not which blocks its records landed in, and a measured
/// need alone cannot tell whether planning counted the entries discovery
/// charged before it.
pub const IDLE_NEED: u64 = 166;

/// What the released world needs: one release completed above the
/// checkpoint and its successor pending.
/// Every root read costs one entry, found or not, and pays for the pages
/// under it. Among them: the successor root the candidate probe finds absent
/// (1); the historical roots a blob record is looked up under (3); the
/// historical drop's result, source, retained-candidate and dropped-record
/// roots (4); and the redo's source and candidate roots (2), once each, their
/// inventories paid by the same entry.
pub const RELEASED_NEED: u64 = 176;

/// What the first reopen needs where two releases completed above a
/// checkpoint that heads the first of them, and its third batch is pending:
/// the retirements among them differ in the free extents they leave.
/// Every root read costs one entry, found or not, and pays for the pages
/// under it. Among them: the successor root the candidate probe finds absent
/// (1); the historical roots a blob record is looked up under (10); the two
/// historical drops' result, source, retained-candidate and dropped-record
/// roots (8); and their redos' source and candidate roots (4), once each,
/// their inventories paid by the same entry.
pub const ORDERED_NEED: u64 = 396;

/// A release completed above the checkpoint: the walk runs for its drop,
/// which then reads the source root it released from.
pub fn released_world() -> PendingWalWorld {
    let world = pending_wal_world::first();
    world.kill_successor_of_first_object();
    world
}

/// One entry short of the need is that limit, and the refused charge is the
/// one that reaches the need. The blocked attempt has no effect.
pub fn assert_one_entry_short_reports_the_limit(world: &PendingWalWorld, need: u64, stage: &str) {
    let admitted = need - 1;
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        crate::production_entry::certified_release_serving::recovery_request::request_with_manifest_entries(world.root(), admitted),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("{stage}: {admitted} entries must block: {outcome:?}")
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    // Discovery charged entries of the same limit before planning.
    assert!(
        (1..admitted).contains(&evidence.counters.manifest_entries),
        "{stage}: discovery charged {} of {admitted}",
        evidence.counters.manifest_entries,
    );
    assert_eq!(
        blocked.cause().limit().map(|limit| (
            limit.dimension(),
            limit.observed(),
            limit.admitted()
        )),
        Some((
            worth_store_recovery_runtime::PhysicalRecoveryLimitDimension::ManifestEntries,
            need,
            admitted
        )),
        "{stage}: denial={:?}",
        evidence.planning_denial,
    );
}

/// Recovers under exactly the need. The blocked attempt left the media as
/// the kill did.
pub fn serve_under(world: &PendingWalWorld, need: u64, stage: &str) -> ServingPhysicalRuntime {
    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(
        crate::production_entry::certified_release_serving::recovery_request::request_with_manifest_entries(world.root(), need),
    );
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage}: {need} entries must recover: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("recovered custody seal");
    crate::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal)
}
