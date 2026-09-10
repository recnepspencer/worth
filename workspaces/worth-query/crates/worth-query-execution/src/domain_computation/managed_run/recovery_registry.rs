//! Instance-local recovery-handle live set (R8.29 / Q8.9).
//!
//! Owned by one application runtime and stamped with that runtime's authority
//! identity at construction. Registry slots are membership tokens local to one
//! instance — they are not cross-runtime authority. Slot allocation still
//! starts at `1` in every instance; recovery effect/inspect authority binds to
//! the runtime identity basis instead (Q8.20).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Authoritative commit identity claimed exactly once for recovery minting.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct WorthQueryRecoveryMintClaim {
    provider_runtime_instance_id: u64,
    product_commit: CompositeCommitIdentity,
}

impl WorthQueryRecoveryMintClaim {
    pub(crate) fn new(
        provider_runtime_instance_id: u64,
        product_commit: CompositeCommitIdentity,
    ) -> Self {
        Self {
            provider_runtime_instance_id,
            product_commit,
        }
    }
}

/// The receipt's authoritative commit identity was already claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryRecoveryMintAlreadyClaimed;

/// Terminal fate recorded when a live recovery resource leaves the registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryRecoveryResourceTerminal {
    Consumed,
    Expired,
    Disposed,
    ForceTerminated,
    /// The handle left without a transition because the attempt was denied.
    ///
    /// Distinct from `Disposed` on purpose. Disposal is a decision — the holder
    /// chose to end recovery. Relinquishment is a non-event: authority was
    /// wrong or stale, nothing was consumed, and the commit is exactly as
    /// recoverable as it was a moment earlier. Only this fate releases the
    /// authoritative mint claim (Q8.21-L11).
    Relinquished,
}

impl worth_proof::TerminalState for WorthQueryRecoveryResourceTerminal {
    fn label(&self) -> &'static str {
        match self {
            Self::Consumed => "consumed",
            Self::Expired => "expired",
            Self::Disposed => "disposed",
            Self::ForceTerminated => "force-terminated",
            Self::Relinquished => "relinquished",
        }
    }
}

/// Opaque registry slot. Not a public recovery identity and not Clone on the
/// handle path — only the registry retains membership.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct WorthQueryRecoveryRegistrySlot(u64);

impl WorthQueryRecoveryRegistrySlot {
    #[cfg(test)]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0
    }
}

struct RecoveryRegistryState {
    next_slot: u64,
    claimed_commits: HashSet<WorthQueryRecoveryMintClaim>,
    /// Which live slot holds which claim, so a relinquished attempt can give
    /// its claim back. Only `register_once` adds; every exit path removes.
    claims_by_slot: HashMap<WorthQueryRecoveryRegistrySlot, WorthQueryRecoveryMintClaim>,
    live: HashMap<WorthQueryRecoveryRegistrySlot, ()>,
    terminated: HashMap<WorthQueryRecoveryRegistrySlot, WorthQueryRecoveryResourceTerminal>,
}

impl RecoveryRegistryState {
    fn new() -> Self {
        Self {
            // Instance-local membership only. Cross-runtime uniqueness lives on
            // the registry's runtime authority identity, not this counter.
            next_slot: 1,
            claimed_commits: HashSet::new(),
            claims_by_slot: HashMap::new(),
            live: HashMap::new(),
            terminated: HashMap::new(),
        }
    }
}

/// Framework-owned recovery resource registry (managed-run family).
///
/// One instance per application runtime. Handles hold an `Arc` to the same
/// registry so Drop/consume remain correct without a process-global table.
///
/// The registry also carries the owning runtime's authorization clock. That is
/// what lets a handle re-check its own deadline without any transition growing
/// a clock parameter — freshness stays something the runtime samples, never
/// something a caller presents (R8.31).
pub(crate) struct WorthQueryRecoveryHandleRegistry {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    clock: Arc<WorthQueryRuntimeClock>,
    state: Mutex<RecoveryRegistryState>,
}

impl std::fmt::Debug for WorthQueryRecoveryHandleRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryRecoveryHandleRegistry")
            .field("runtime_authority", &self.runtime_authority)
            .finish_non_exhaustive()
    }
}

impl WorthQueryRecoveryHandleRegistry {
    /// Bind this registry to one concrete execution-runtime authority identity
    /// and that runtime's authorization clock.
    pub(crate) fn for_runtime(
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        clock: Arc<WorthQueryRuntimeClock>,
    ) -> Self {
        Self {
            runtime_authority,
            clock,
            state: Mutex::new(RecoveryRegistryState::new()),
        }
    }

    /// Test fixture registry with a freshly minted runtime authority identity.
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::for_runtime(
            WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
            Arc::new(WorthQueryRuntimeClock::system()),
        )
    }

    /// Runtime authority identity this registry belongs to (Q8.20).
    pub(crate) fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    /// The owning runtime's authorization clock (R8.31). Not a caller lane —
    /// `WorthQueryRuntimeClock` is crate-private and samples itself.
    pub(crate) fn clock(&self) -> &WorthQueryRuntimeClock {
        &self.clock
    }

    /// Atomically claim one commit and register its sole recovery resource.
    pub(crate) fn register_once(
        &self,
        claim: WorthQueryRecoveryMintClaim,
    ) -> Result<WorthQueryRecoveryRegistrySlot, WorthQueryRecoveryMintAlreadyClaimed> {
        let mut state = self.state.lock().expect("recovery registry lock");
        if !state.claimed_commits.insert(claim.clone()) {
            return Err(WorthQueryRecoveryMintAlreadyClaimed);
        }
        let slot = WorthQueryRecoveryRegistrySlot(state.next_slot);
        state.next_slot = state.next_slot.saturating_add(1);
        state.claims_by_slot.insert(slot, claim);
        state.live.insert(slot, ());
        Ok(slot)
    }

    /// Retire a slot *without* consuming the recovery it stands for.
    ///
    /// The mint claim goes back, so the receipt — cloneable historical evidence
    /// the holder still has — can mint a fresh handle and the attempt can be
    /// made again with correct authority. The slot still leaves the live set
    /// exactly once, atomically, under the same lock that releases the claim.
    ///
    /// Returns `false` for a slot that is not live, which is what keeps an
    /// already-terminal handle from resurrecting a spent commit: `ensure_live`
    /// denies, the denial relinquishes, and there is nothing to give back.
    ///
    /// Unlike [`Self::mark_terminal`] this records no `terminated` entry. A
    /// relinquished slot is unobservable — its handle is gone, `terminal_of` is
    /// reached only from `consume` and `Drop`, and a retry gets a fresh slot —
    /// so an entry would be pure residue. It would also be residue a caller
    /// controls: retrying a denied transition in a loop is the one path that can
    /// produce unboundedly many slots for a single commit.
    pub(crate) fn relinquish(&self, slot: WorthQueryRecoveryRegistrySlot) -> bool {
        let mut state = self.state.lock().expect("recovery registry lock");
        if state.live.remove(&slot).is_none() {
            return false;
        }
        if let Some(claim) = state.claims_by_slot.remove(&slot) {
            state.claimed_commits.remove(&claim);
        }
        true
    }

    /// Register an unclaimed slot for owner-level lifecycle fixtures.
    #[cfg(test)]
    pub(crate) fn register_axis_probe(&self) -> WorthQueryRecoveryRegistrySlot {
        let mut state = self.state.lock().expect("recovery registry lock");
        let slot = WorthQueryRecoveryRegistrySlot(state.next_slot);
        state.next_slot = state.next_slot.saturating_add(1);
        state.live.insert(slot, ());
        slot
    }

    /// Liveness of a slot the asker already holds.
    ///
    /// This observation remains inside the execution owner.
    pub(crate) fn is_live(&self, slot: WorthQueryRecoveryRegistrySlot) -> bool {
        self.state
            .lock()
            .expect("recovery registry lock")
            .live
            .contains_key(&slot)
    }

    /// Every live slot in this instance — including slots belonging to handles
    /// the asker does not hold. This is what turns a slot-addressed terminal
    /// into a cross-holder one, so it is fixture-only; no production path needs
    /// to ask the question, and clippy confirms it (the method is dead code in
    /// a build without this gate).
    #[cfg(test)]
    pub(crate) fn enumerate_live(&self) -> Vec<WorthQueryRecoveryRegistrySlot> {
        self.state
            .lock()
            .expect("recovery registry lock")
            .live
            .keys()
            .copied()
            .collect()
    }

    /// Retire a slot and record why.
    ///
    /// Slot-addressed rather than handle-addressed, and the terminal kind is
    /// the caller's to choose — so this is a privileged operation taking an
    /// identity instead of the resource. Crate-private: the only production
    /// callers are `WorthQueryRecoveryHandle::consume` and its `Drop`, both of
    /// which hold the handle itself and name the terminal the runtime caused.
    pub(crate) fn mark_terminal(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
        terminal: WorthQueryRecoveryResourceTerminal,
    ) -> bool {
        let mut state = self.state.lock().expect("recovery registry lock");
        if state.live.remove(&slot).is_some() {
            // The claim itself stays in `claimed_commits`: this slot reached a
            // real terminal, so the commit's one recovery was exercised and no
            // second handle may be minted for it. Only `relinquish` gives a
            // claim back.
            state.claims_by_slot.remove(&slot);
            state.terminated.insert(slot, terminal);
            true
        } else {
            false
        }
    }

    pub(crate) fn terminal_of(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
    ) -> Option<WorthQueryRecoveryResourceTerminal> {
        self.state
            .lock()
            .expect("recovery registry lock")
            .terminated
            .get(&slot)
            .copied()
    }

    /// Leak detection: no live handle may remain after the named terminals.
    #[cfg(test)]
    pub(crate) fn assert_no_live_handles(&self) {
        let live = self.enumerate_live();
        assert!(
            live.is_empty(),
            "recovery handles leaked from terminal paths: {live:?}"
        );
    }
}
