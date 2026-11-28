//! Durable root pins held beside live read registrations.
//!
//! A pin protects one published root generation exactly as a live reader of
//! that root does: displaced segments of that root and its inline tail, and
//! displaced extents and arenas at or above it. The serving runtime replays
//! the pin journal before it is handed out, so no retirement can run ahead of
//! restoration; a journal that cannot be replayed leaves the table
//! unrestored, and an unrestored table protects every root.
//!
//! The state mutex is a leaf: it is never held across I/O or while acquiring
//! the current-root, registration or reclamation locks.
#[cfg(test)]
mod tests;

use std::sync::{Mutex, MutexGuard};

use worth_store_physical_format::DurablePhysicalRootManifest;

use crate::pin_table::{
    RootPinCore, RootPinDenial, RootPinDropPolicy, RootPinExpiry, RootPinId, RootPinState,
};

/// What one pin protects and returns to its loans.
pub(in crate::physical_runtime) struct PinnedPhysicalRoot {
    /// The pinned root, when this process holds its manifest.
    pub(in crate::physical_runtime) manifest: Option<DurablePhysicalRootManifest>,
    pub(in crate::physical_runtime) inline_tail: Option<(u64, u64)>,
    /// Opaque owner bytes recorded with the pin.
    pub(in crate::physical_runtime) payload: Vec<u8>,
}

impl PinnedPhysicalRoot {
    pub(in crate::physical_runtime) fn of(
        manifest: Option<DurablePhysicalRootManifest>,
        inline_tail: Option<(u64, u64)>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            manifest,
            inline_tail,
            payload,
        }
    }
}

pub(in crate::physical_runtime) fn inline_tail_of(
    root: &DurablePhysicalRootManifest,
) -> Option<(u64, u64)> {
    root.last_inline_segment()
        .map(|cell| (cell.segment_id().get(), cell.generation().get()))
}

pub(in crate::physical_runtime) struct RootPinTableState {
    restored: bool,
    core: RootPinCore<PinnedPhysicalRoot>,
    /// Prior root manifests loaded by open, kept only until restoration.
    staged: Vec<DurablePhysicalRootManifest>,
}

pub(in crate::physical_runtime) struct RootPinTable {
    state: Mutex<RootPinTableState>,
    journal: Mutex<()>,
}

impl Default for RootPinTable {
    fn default() -> Self {
        Self {
            state: Mutex::new(RootPinTableState {
                restored: true,
                core: RootPinCore::default(),
                staged: Vec::new(),
            }),
            journal: Mutex::new(()),
        }
    }
}

impl RootPinTable {
    fn lock(&self) -> MutexGuard<'_, RootPinTableState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Serializes pin journal appends. Taken before the current-root lock and
    /// never while holding the state mutex.
    pub(in crate::physical_runtime) fn lock_journal(&self) -> MutexGuard<'_, ()> {
        self.journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Keep the prior roots open loaded until the pin journal is replayed.
    pub(in crate::physical_runtime) fn stage_prior_roots(
        &self,
        roots: Vec<DurablePhysicalRootManifest>,
    ) {
        self.lock().staged = roots;
    }

    /// The pin journal could not be replayed: protect every root.
    pub(in crate::physical_runtime) fn mark_unrestored(&self) {
        let mut state = self.lock();
        state.restored = false;
        state.staged = Vec::new();
    }

    /// Replace the table with `restored` and release staged manifests.
    pub(in crate::physical_runtime) fn install_restored(
        &self,
        restored: RootPinCore<PinnedPhysicalRoot>,
    ) {
        *self.lock() = RootPinTableState {
            restored: true,
            core: restored,
            staged: Vec::new(),
        };
    }

    /// Take the staged prior roots for restoration.
    pub(in crate::physical_runtime) fn take_staged(&self) -> Vec<DurablePhysicalRootManifest> {
        std::mem::take(&mut self.lock().staged)
    }

    pub(in crate::physical_runtime) fn restored(&self) -> bool {
        self.lock().restored
    }

    fn with_restored<R>(
        &self,
        apply: impl FnOnce(&mut RootPinCore<PinnedPhysicalRoot>) -> Result<R, RootPinDenial>,
    ) -> Result<R, RootPinDenial> {
        let mut state = self.lock();
        if !state.restored {
            return Err(RootPinDenial::PinsUnavailable);
        }
        apply(&mut state.core)
    }

    fn protects(&self, query: impl FnOnce(&RootPinCore<PinnedPhysicalRoot>) -> bool) -> bool {
        let state = self.lock();
        // Unrestored pins protect everything.
        !state.restored || query(&state.core)
    }

    pub(in crate::physical_runtime) fn protects_root_at_or_below(&self, generation: u64) -> bool {
        self.protects(|core| core.protects_root_at_or_below(generation))
    }

    /// Projects every pin that reaches `window`; unrestored pins reach all.
    pub(in crate::physical_runtime) fn project_reach(
        &self,
        window: crate::physical_runtime::durability::ReachWindow,
        reach: &mut crate::physical_runtime::durability::ProtectedReachSet,
    ) {
        use crate::physical_runtime::durability::ReachWindow;
        let state = self.lock();
        if !state.restored {
            reach.protect_all_roots();
            return;
        }
        match window {
            ReachWindow::Root {
                generation,
                inline_tail,
            } => {
                if state.core.protects_root(generation) {
                    reach.protect_root(generation);
                }
                if state
                    .core
                    .any_live(|root| root.inline_tail == Some(inline_tail))
                {
                    reach.protect_inline_tail(inline_tail.0, inline_tail.1);
                }
            }
            ReachWindow::RootsAtOrBelow(generation) => {
                if let Some(root) = state.core.protected_root_at_or_below(generation) {
                    reach.protect_root(root);
                }
            }
        }
    }

    /// Pin `root` in memory. The caller holds the current-root lock.
    pub(in crate::physical_runtime) fn pin(
        &self,
        root: &DurablePhysicalRootManifest,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        payload: Vec<u8>,
    ) -> Result<RootPinId, RootPinDenial> {
        let pinned = PinnedPhysicalRoot::of(Some(root.clone()), inline_tail_of(root), payload);
        self.with_restored(|core| core.acquire(root.generation(), parent, policy, pinned))
    }

    /// Undo an in-memory pin whose durable record proved to have no effect.
    pub(in crate::physical_runtime) fn withdraw(&self, pin: RootPinId) {
        let _ = self.with_restored(|core| {
            core.expire(pin)?;
            core.expire(pin)
        });
    }

    /// Admit one reader loan; returns the pinned manifest and owner bytes.
    pub(in crate::physical_runtime) fn loan(
        &self,
        pin: RootPinId,
    ) -> Result<(Option<DurablePhysicalRootManifest>, Vec<u8>), RootPinDenial> {
        self.with_restored(|core| {
            core.loan(pin)
                .map(|root| (root.manifest.clone(), root.payload.clone()))
        })
    }

    pub(in crate::physical_runtime) fn release_loan(&self, pin: RootPinId) {
        let _ = self.with_restored(|core| {
            core.release_loan(pin);
            Ok(())
        });
    }

    pub(in crate::physical_runtime) fn expiry(
        &self,
        pin: RootPinId,
    ) -> Result<RootPinExpiry, RootPinDenial> {
        self.with_restored(|core| core.expiry(pin))
    }

    pub(in crate::physical_runtime) fn expire(
        &self,
        pin: RootPinId,
    ) -> Result<RootPinState, RootPinDenial> {
        self.with_restored(|core| core.expire(pin))
    }

    pub(in crate::physical_runtime) fn parent(&self, pin: RootPinId) -> Option<RootPinId> {
        self.with_restored(|core| Ok(core.parent(pin)))
            .ok()
            .flatten()
    }

    pub(in crate::physical_runtime) fn state(&self, pin: RootPinId) -> Option<RootPinState> {
        self.with_restored(|core| Ok(core.state(pin)))
            .ok()
            .flatten()
    }

    pub(in crate::physical_runtime) fn root(&self, pin: RootPinId) -> Option<u64> {
        self.with_restored(|core| Ok(core.root(pin))).ok().flatten()
    }

    pub(in crate::physical_runtime) fn payload(&self, pin: RootPinId) -> Option<Vec<u8>> {
        self.with_restored(|core| Ok(core.live_attachment(pin).map(|root| root.payload.clone())))
            .ok()
            .flatten()
    }

    /// The position an ordered journal held in a live pin's root, when this
    /// process holds that root's manifest.
    pub(in crate::physical_runtime) fn journal_head(
        &self,
        pin: RootPinId,
        journal: worth_store_physical_format::RecordAnchorName,
    ) -> Option<u64> {
        self.with_restored(|core| {
            Ok(core
                .live_attachment(pin)
                .and_then(|root| root.manifest.as_ref())
                .map(|manifest| manifest.record_anchors().generation(journal)))
        })
        .ok()
        .flatten()
    }

    pub(in crate::physical_runtime) fn drained(&self) -> Result<Vec<RootPinId>, RootPinDenial> {
        self.with_restored(|core| Ok(core.drained()))
    }

    pub(in crate::physical_runtime) fn live_pins(&self) -> usize {
        self.lock().core.live_pins()
    }

    pub(in crate::physical_runtime) fn live_pin_ids(&self) -> Vec<RootPinId> {
        self.with_restored(|core| Ok(core.live_pin_ids()))
            .unwrap_or_default()
    }
}
