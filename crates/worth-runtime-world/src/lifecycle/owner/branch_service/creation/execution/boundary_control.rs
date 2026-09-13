//! Test-only rendezvous at the between-owners boundary of one creation.
//!
//! The creation path rechecks the product head between its two owner forks.
//! Reaching that recheck with a moved head requires another operation to
//! settle after the Relational fork and before the recheck, which no budget or
//! intent can arrange on its own. This control stops exactly one armed
//! creation there and hands its owner identity to the arming test, which moves
//! the head and releases it.

use std::sync::mpsc::SyncSender;

use crate::identity::RuntimeWorldOwnerIdentity;
use crate::lifecycle::owner::rehearsal::{OwnerRehearsalGuard, OwnerRehearsalRegistry};
use crate::lifecycle::RuntimeWorldOwnerRoot;

static SIGNAL_CUTOFF: OwnerRehearsalRegistry = OwnerRehearsalRegistry::new("Signal fork cutoff");

static CREATION_BOUNDARY: OwnerRehearsalRegistry = OwnerRehearsalRegistry::new("creation boundary");

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Pause after Signal reserves the real fork, before its cancellation cutoff.
    pub(crate) fn rehearse_signal_fork_cutoff(
        &self,
        reached: SyncSender<RuntimeWorldOwnerIdentity>,
    ) -> OwnerRehearsalGuard {
        SIGNAL_CUTOFF.arm(self.owner_identity(), reached)
    }

    /// Arm one creation on this owner to stop after its Relational fork and
    /// before the head recheck that guards the Signal fork. The owner identity
    /// of the paused creation arrives on `reached`; dropping the returned guard
    /// releases the creation and disarms the rehearsal.
    pub(crate) fn rehearse_creation_fork_boundary(
        &self,
        reached: SyncSender<RuntimeWorldOwnerIdentity>,
    ) -> OwnerRehearsalGuard {
        CREATION_BOUNDARY.arm(self.owner_identity(), reached)
    }
}

/// Hold the single armed creation between its two owner forks. Every other
/// creation passes straight through.
pub(super) fn pause_between_creation_forks(owner: RuntimeWorldOwnerIdentity) {
    CREATION_BOUNDARY.pause(owner);
}

pub(super) fn pause_at_signal_fork_cutoff(owner: RuntimeWorldOwnerIdentity) {
    SIGNAL_CUTOFF.pause(owner);
}
