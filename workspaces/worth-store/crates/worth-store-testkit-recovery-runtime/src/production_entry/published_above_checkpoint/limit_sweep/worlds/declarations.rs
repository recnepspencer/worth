//! The worlds the sweep takes as their kill or their last close left them.
//! Each is built afresh for every swept limit: a recovery changes it.

pub use {crate::production_entry::published_above_checkpoint::limit_sweep::worlds::Killed, std::path::Path, crate::production_entry::pending_wal_world::PendingWalWorld, worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_proof::TransitionOutcome, crate::production_entry::certified_release_serving, crate::production_entry::pending_successor_above_history, crate::production_entry::release_reopen, crate::production_entry::published_above_checkpoint::manifest_entry_limit::released_world, crate::production_entry::tier_release_pending_wal};
pub use {worth_store_test_support::TemporaryDirectory};

/// A world whose media the sweep recovers; the sweep reads only its root.
pub trait Killed {
    fn root(&self) -> &Path;
}

impl Killed for PendingWalWorld {
    fn root(&self) -> &Path {
        crate::production_entry::pending_wal_world::PendingWalWorld::root(self)
    }
}

impl Killed for TemporaryDirectory {
    fn root(&self) -> &Path {
        self.path()
    }
}

impl Killed for crate::production_entry::tier_release_pending_wal::KilledWorld {
    fn root(&self) -> &Path {
        crate::production_entry::tier_release_pending_wal::KilledWorld::root(self)
    }
}

/// A release whose every batch completed, checkpointed by the Serving that
/// released it and then closed: recovery admits the certified head and opens
/// nothing pending.
pub fn certified_release() -> TemporaryDirectory {
    let (world, receipt, _) = crate::production_entry::release_reopen::released_world(1024);
    assert_eq!(receipt.remaining_payload_records(), 0);
    let request = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x7c; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("release checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    drop(world);
    retained
}

/// The released world after its pending successor was recovered, sealed and
/// served: the next recovery admits that successor as completed history above
/// the same checkpoint.
pub fn completed_history() -> PendingWalWorld {
    let world = released_world();
    let seal = crate::production_entry::pending_successor_above_history::recover(&world, "pending successor")
        .into_core()
        .certification_release_media_into_checkpoint_custody()
        .expect("Store must seal the pending successor");
    crate::production_entry::certified_release_serving::admit_serving_with_seal(world.root(), seal).close();
    world
}