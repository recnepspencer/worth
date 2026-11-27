//! The capability every full-history lineage scan requires.
//!
//! Rebuilding derived naming, group or index state from published events is
//! only sound when the state holds its complete canonical event history. A
//! [`FullLineageRecoveryToken`] is that proof: it is minted only where the
//! state is full by construction (a fresh runtime that has seen every event
//! since genesis, and the full canonical replay that installs every event at
//! once), and it is dropped the moment cold pages or a cold naming base are
//! installed. Its constructor is private to this module, so a cold
//! checkpoint reopen cannot name, mint or receive one, and every full scan
//! (`rebuild_derived_indexes`, branch naming, group and index rebuilds) is
//! statically unreachable from it.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_STATE_ID: AtomicU64 = AtomicU64::new(1);

/// Opaque runtime-unique owner identity for one [`super::LineageState`].
/// Its `Debug` output is intentionally constant so whole-runtime state-equality
/// snapshots across recovery retries do not spuriously differ on process-global IDs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct LineageStateId(u64);

impl fmt::Debug for LineageStateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LineageStateId")
    }
}

pub(super) fn next_state_id() -> LineageStateId {
    LineageStateId(NEXT_STATE_ID.fetch_add(1, Ordering::Relaxed))
}

/// Proof that a specific lineage state holds its complete canonical event
/// history. Non-`Copy` and bound to its owning state's identity so it cannot
/// be cloned or forged across states.
#[derive(Debug, PartialEq, Eq)]
pub struct FullLineageRecoveryToken {
    state_id: LineageStateId,
}

impl FullLineageRecoveryToken {
    /// A fresh state: nothing has been published, so nothing is missing.
    pub(super) const fn fresh(state_id: LineageStateId) -> Self {
        Self { state_id }
    }

    /// A full canonical replay: the caller installs every published event.
    pub(super) const fn full_replay(state_id: LineageStateId) -> Self {
        Self { state_id }
    }

    /// Verifies that this token belongs to `state_id`.
    pub(super) fn assert_owner(&self, state_id: LineageStateId) {
        assert_eq!(
            self.state_id, state_id,
            "FullLineageRecoveryToken used against a different LineageState"
        );
    }
}

impl super::LineageState {
    /// Returns this state's full-history proof when every event since genesis
    /// is resident; `None` once cold pages or a cold naming base are installed.
    pub fn full_history_token(&self) -> Option<&FullLineageRecoveryToken> {
        self.full_history.as_ref()
    }

    /// Rebuilds every derived index using this state's own recovery token when
    /// full resident history is held; returns `false` on a cold-backed state.
    pub fn rebuild_derived_indexes_if_full(
        &mut self,
        lane: &crate::contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> bool {
        let Some(full) = self.full_history.take() else {
            return false;
        };
        self.rebuild_derived_indexes(lane, &full);
        self.full_history = Some(full);
        true
    }
}
