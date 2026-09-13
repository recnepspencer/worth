//! Test-only rendezvous before a source-guarded installation.
//!
//! Exact reuse and fork finalization both install their product reference
//! under the source cell's guard. Reaching that installation with a source
//! head that moved after the last pre-effect recheck needs another operation
//! to publish in that window, which no budget or intent can arrange on its
//! own. This control stops exactly one armed creation just before it takes
//! the guard and hands its owner identity to the arming test, which moves the
//! head and releases it.

use std::sync::mpsc::SyncSender;

use crate::identity::RuntimeWorldOwnerIdentity;
use crate::lifecycle::owner::rehearsal::{OwnerRehearsalGuard, OwnerRehearsalRegistry};
use crate::lifecycle::RuntimeWorldOwnerRoot;

static SOURCE_GUARDED_INSTALL: OwnerRehearsalRegistry =
    OwnerRehearsalRegistry::new("source-guarded install");

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Arm one creation on this owner to stop just before it installs its
    /// product reference under the source guard. The owner identity of the
    /// paused creation arrives on `reached`; dropping the returned guard
    /// releases the creation and disarms the rehearsal.
    pub(crate) fn rehearse_source_guarded_install(
        &self,
        reached: SyncSender<RuntimeWorldOwnerIdentity>,
    ) -> OwnerRehearsalGuard {
        SOURCE_GUARDED_INSTALL.arm(self.owner_identity(), reached)
    }
}

/// Hold the single armed creation before its source-guarded installation.
/// Every other creation passes straight through.
pub(super) fn pause_before_source_guarded_install(owner: RuntimeWorldOwnerIdentity) {
    SOURCE_GUARDED_INSTALL.pause(owner);
}
