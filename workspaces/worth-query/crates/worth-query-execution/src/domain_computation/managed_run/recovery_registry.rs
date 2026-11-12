//! Instance-local recovery-handle live set (R8.29 / Q8.9).
//!
//! Owned by one application runtime and stamped with that runtime's authority
//! identity at construction. Registry slots are membership tokens local to one
//! instance â€” they are not cross-runtime authority. Slot allocation still
//! starts at `1` in every instance; recovery effect/inspect authority binds to
//! the runtime identity basis instead (Q8.20).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
use worth_foundational::facade::RecoveryClaimDisposition;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Authoritative commit identity claimed exactly once for recovery minting.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum WorthQueryRecoveryMintClaim {
    Home(worth_foundational::facade::DurableCompositeCommitIdentity),
    Memory {
        provider_runtime_instance_id: u64,
        world_owner: u64,
        commit_ordinal: u64,
    },
}
impl WorthQueryRecoveryMintClaim {
    pub(crate) fn new(
        provider_runtime_instance_id: u64,
        product_commit: CompositeCommitIdentity,
    ) -> Self {
        match product_commit.owner_identity().durable() {
            Some(owner) => Self::Home(
                worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                    owner,
                    product_commit.ordinal(),
                ),
            ),
            None => Self::Memory {
                provider_runtime_instance_id,
                world_owner: product_commit.owner_identity().get(),
                commit_ordinal: product_commit.ordinal(),
            },
        }
    }
}

/// The receipt's authoritative commit identity was already claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryRecoveryMintAlreadyClaimed;

/// Opaque registry slot. Not a public recovery identity and not Clone on the
/// handle path — only the registry retains membership.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryRecoveryRegistrySlot(u64);

impl WorthQueryRecoveryRegistrySlot {
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
    terminal_by_claim: HashMap<WorthQueryRecoveryMintClaim, RecoveryClaimDisposition>,
    live: BTreeMap<WorthQueryRecoveryRegistrySlot, ()>,
    durable: HashMap<WorthQueryRecoveryRegistrySlot, crate::domain_computation::application_aftermath::dispatch_journal::QueryRecordedRecoveryClaim>,
    deferred: std::collections::BTreeMap<worth_foundational::facade::DurableRecoveryClaimIdentity, DeferredRecoveryDisposition>,
    terminated: HashMap<WorthQueryRecoveryRegistrySlot, RecoveryClaimDisposition>,
}

impl RecoveryRegistryState {
    fn new() -> Self {
        Self {
            // Instance-local membership only. Cross-runtime uniqueness lives on
            // the registry's runtime authority identity, not this counter.
            next_slot: 1,
            claimed_commits: HashSet::new(),
            claims_by_slot: HashMap::new(),
            terminal_by_claim: HashMap::new(),
            live: BTreeMap::new(),
            durable: HashMap::new(),
            deferred: Default::default(),
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
/// a clock parameter â€” freshness stays something the runtime samples, never
/// something a caller presents (R8.31).
pub struct WorthQueryRecoveryHandleRegistry {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    clock: Arc<WorthQueryRuntimeClock>,
    state: Mutex<RecoveryRegistryState>,
    journal: std::sync::Weak<crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal>,
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
        journal: std::sync::Weak<crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal>,
    ) -> Self {
        Self {
            runtime_authority,
            clock,
            journal,
            state: Mutex::new(RecoveryRegistryState::new()),
        }
    }

    /// Test fixture registry with a freshly minted runtime authority identity.
    pub(crate) fn new() -> Self {
        Self::for_runtime(
            WorthQueryRuntimeAuthorityIdentity::mint_for_test(),
            Arc::new(WorthQueryRuntimeClock::system()),
            std::sync::Weak::new(),
        )
    }

    /// Runtime authority identity this registry belongs to (Q8.20).
    pub(crate) fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    /// The owning runtime's authorization clock (R8.31). Not a caller lane â€”
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

    pub(crate) fn claim_state(
        &self,
        claim: &WorthQueryRecoveryMintClaim,
    ) -> Result<(bool, Option<RecoveryClaimDisposition>), crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial>{
        if let Some(recorded) = self.journal_claim_state(claim)? {
            return Ok(recorded);
        }
        let state = self.state.lock().expect("recovery registry lock");
        Ok((
            state.claimed_commits.contains(claim),
            state.terminal_by_claim.get(claim).copied(),
        ))
    }

    /// Retire a slot *without* consuming the recovery it stands for.
    ///
    /// The mint claim goes back, so the receipt â€” cloneable historical evidence
    /// the holder still has â€” can mint a fresh handle and the attempt can be
    /// made again with correct authority. The slot still leaves the live set
    /// exactly once, atomically, under the same lock that releases the claim.
    ///
    /// Returns `false` for a slot that is not live, which is what keeps an
    /// already-terminal handle from resurrecting a spent commit: `ensure_live`
    /// denies, the denial relinquishes, and there is nothing to give back.
    ///
    /// Unlike [`Self::mark_terminal`] this records no `terminated` entry. A
    /// relinquished slot is unobservable â€” its handle is gone, `terminal_of` is
    /// reached only from `consume` and `Drop`, and a retry gets a fresh slot â€”
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
        if let Some(recorded) = state.durable.remove(&slot) {
            state
                .deferred
                .entry(recorded.identity)
                .or_insert(DeferredRecoveryDisposition {
                    recorded,
                    terminal: RecoveryClaimDisposition::Relinquished,
                });
        }
        true
    }

    /// Register an unclaimed slot for owner-level lifecycle fixtures.
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

    /// Every live slot in this instance â€” including slots belonging to handles
    /// the asker does not hold. This is what turns a slot-addressed terminal
    /// into a cross-holder one, so it is fixture-only; no production path needs
    /// to ask the question, and clippy confirms it (the method is dead code in
    /// a build without this gate).
    pub(crate) fn enumerate_live(&self) -> Vec<WorthQueryRecoveryRegistrySlot> {
        let mut slots: Vec<_> = self
            .state
            .lock()
            .expect("recovery registry lock")
            .live
            .keys()
            .copied()
            .collect();
        slots.sort_unstable_by_key(|slot| slot.0);
        slots
    }

    /// Retire a slot and record why.
    ///
    /// Slot-addressed rather than handle-addressed, and the terminal kind is
    /// the caller's to choose â€” so this is a privileged operation taking an
    /// identity instead of the resource. Crate-private: the only production
    /// callers are `WorthQueryRecoveryHandle::consume` and its `Drop`, both of
    /// which hold the handle itself and name the terminal the runtime caused.
    pub(crate) fn mark_terminal(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
        terminal: RecoveryClaimDisposition,
    ) -> bool {
        let mut state = self.state.lock().expect("recovery registry lock");
        if state.live.remove(&slot).is_some() {
            // The claim itself stays in `claimed_commits`: this slot reached a
            // real terminal, so the commit's one recovery was exercised and no
            // second handle may be minted for it. Only `relinquish` gives a
            // claim back.
            if let Some(claim) = state.claims_by_slot.remove(&slot) {
                state.terminal_by_claim.insert(claim, terminal);
            }
            if let Some(recorded) = state.durable.remove(&slot) {
                state
                    .deferred
                    .entry(recorded.identity)
                    .or_insert(DeferredRecoveryDisposition { recorded, terminal });
            }
            state.terminated.insert(slot, terminal);
            true
        } else {
            false
        }
    }

    pub(crate) fn terminal_of(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
    ) -> Option<RecoveryClaimDisposition> {
        self.state
            .lock()
            .expect("recovery registry lock")
            .terminated
            .get(&slot)
            .copied()
    }

    /// Leak detection: no live handle may remain after the named terminals.
    pub(crate) fn assert_no_live_handles(&self) {
        let live = self.enumerate_live();
        assert!(
            live.is_empty(),
            "recovery handles leaked from terminal paths: {live:?}"
        );
    }
}

mod court_doors_for_test;
mod recorded_observation;

mod durable;
pub(in crate::domain_computation) use durable::DeferredRecoveryDisposition;
