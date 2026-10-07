//! The worlds the sweep takes as their kill or their last close left them.
//! Each producer runs once; independent media copies isolate successful sweeps.

use super::*;
use worth_store_test_support::TemporaryDirectory;

/// A world whose media the sweep recovers; the sweep reads only its root.
pub(super) trait Killed {
    fn root(&self) -> &Path;
}

impl Killed for PendingWalWorld {
    fn root(&self) -> &Path {
        PendingWalWorld::root(self)
    }
}

impl Killed for TemporaryDirectory {
    fn root(&self) -> &Path {
        self.path()
    }
}

impl Killed for tier_release_pending_wal::KilledWorld {
    fn root(&self) -> &Path {
        tier_release_pending_wal::KilledWorld::root(self)
    }
}

/// A release whose every batch completed, checkpointed by the Serving that
/// released it and then closed: recovery admits the certified head and opens
/// nothing pending.
pub(super) fn certified_release() -> TemporaryDirectory {
    let (world, receipt, _) = release_reopen::released_world(1024);
    assert_eq!(receipt.remaining_payload_records(), 0);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0x7c; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(request).into_raw()
    else {
        panic!("release checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
    let retained = world.retained_root();
    drop(world);
    retained
}

/// The released world after its pending successor was recovered, sealed and
/// served: the next recovery admits that successor as completed history above
/// the same checkpoint.
pub(super) fn completed_history() -> PendingWalWorld {
    let world = released_world();
    let seal = pending_successor_above_history::recover(&world, "pending successor")
        .into_core()
        .into_checkpoint_custody()
        .expect("Store must seal the pending successor");
    certified_release_serving::admit_serving_with_seal(world.root(), seal).close();
    world
}
