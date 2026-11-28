//! Inspection, residency, pin-settlement, branch-retirement, and
//! certification-hook accessors on [`DurableHistoricalBasisStore`].

use std::sync::Arc;

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{HistoricalReadDenial, HistoricalResidencyPoolSnapshot};
use worth_store::pin_table::{RootPinId, RootPinState};

use super::chunk_format::ChunkMemberId;
use super::durable::DurableHistoricalBasisStore;
use super::durable_io::{
    corrupt, load_basis_manifest_on_demand, pin_write, publish_catalog, DurableMemberReader,
};
use super::durable_reconcile::ensure_reconciled_locked;
#[cfg(feature = "certification-test-authority")]
use super::durable_reconcile::DurableHistoricalCrashCut;
use super::residency::{HeadWorkingSetGuard, HistoricalPoolBorrow};
#[cfg(feature = "certification-test-authority")]
use super::stream::HistoricalCursorCertificationMutant;

impl DurableHistoricalBasisStore {
    pub fn home_identity(&self) -> HomeIdentity {
        self.inner.home_identity
    }

    pub fn home_incarnation(&self) -> HomeIncarnation {
        self.inner.home_incarnation
    }

    pub fn residency_snapshot(&self) -> HistoricalResidencyPoolSnapshot {
        self.inner.residency.snapshot()
    }

    pub fn manifest_pages_faulted(&self) -> u64 {
        self.inner.residency.manifest_pages_faulted()
    }

    pub fn basis_stage_publications(&self) -> u64 {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .basis_stage_generation
    }

    pub fn pin_head_bytes(&self, bytes: u64) -> Option<HeadWorkingSetGuard> {
        self.inner.residency.pin_head_bytes(bytes)
    }

    pub fn borrow_historical_bytes(
        &self,
        bytes: u64,
    ) -> Result<HistoricalPoolBorrow, HistoricalReadDenial> {
        self.inner.residency.borrow_historical_bytes(bytes)
    }

    pub fn discard_auxiliary_indexes(&self) -> usize {
        self.inner.aux_indexes.discard_all()
    }

    pub fn auxiliary_index_count(&self) -> usize {
        self.inner.aux_indexes.index_count()
    }

    pub fn pin_state(&self, commit_id: CommitId) -> Option<RootPinState> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = state.catalog.bases.get(&commit_id)?;
        self.inner
            .serving
            .root_pins()
            .ok()?
            .state(RootPinId::from_ordinal(entry.pin_ordinal))
    }

    pub fn raw_pin_state(&self, pin: RootPinId) -> Option<RootPinState> {
        self.inner.serving.root_pins().ok()?.state(pin)
    }

    pub fn live_pin_count(&self) -> usize {
        self.inner
            .serving
            .root_pins()
            .ok()
            .map_or(0, |pins| pins.live_pins())
    }

    pub fn retained_basis_count(&self) -> usize {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .catalog
            .bases
            .len()
    }

    pub fn pending_retire_pin_count(&self) -> usize {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .catalog
            .pending_retire_pins
            .len()
    }

    pub fn pending_retire_branches(&self) -> Vec<String> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .catalog
            .pending_retire_branches
            .clone()
    }

    pub fn inflight_retain_branches(&self) -> Vec<String> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .catalog
            .inflight_retain_branches
            .clone()
    }

    pub fn is_retained_branch(&self, branch_name: &str) -> bool {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retained_branches
            .contains(branch_name)
    }

    /// Durably records a pre-fork active retain intent BEFORE calling
    /// `fork_preview`. Live reconciliation NEVER touches in-flight intents;
    /// only cold reopen after a crash promotes surviving intents to retirement.
    pub fn record_inflight_retain_intent(
        &self,
        branch_name: &str,
    ) -> Result<(), HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)?;
        if !state
            .catalog
            .inflight_retain_branches
            .iter()
            .any(|name| name == branch_name)
        {
            let mut next = state.catalog.clone();
            next.inflight_retain_branches.push(branch_name.to_owned());
            publish_catalog(&self.inner.serving, &mut state, &next)?;
            state.catalog = next;
        }
        Ok(())
    }

    /// Atomically moves an abandoned/failed in-flight retain intent into
    /// `pending_retire_branches` so retirement closure is retried durably.
    pub fn abandon_inflight_retain_intent(
        &self,
        branch_name: &str,
    ) -> Result<(), HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)?;
        let mut next = state.catalog.clone();
        next.inflight_retain_branches
            .retain(|name| name != branch_name);
        if !next
            .pending_retire_branches
            .iter()
            .any(|name| name == branch_name)
        {
            next.pending_retire_branches.push(branch_name.to_owned());
        }
        if next != state.catalog {
            publish_catalog(&self.inner.serving, &mut state, &next)?;
            state.catalog = next;
        }
        Ok(())
    }

    /// Durably records an explicitly retired/orphaned branch eligible for
    /// immediate `close_branch` reconciliation.
    pub fn record_pending_branch_intent(
        &self,
        branch_name: &str,
    ) -> Result<(), HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)?;
        if !state
            .catalog
            .pending_retire_branches
            .iter()
            .any(|name| name == branch_name)
        {
            let mut next = state.catalog.clone();
            next.pending_retire_branches.push(branch_name.to_owned());
            publish_catalog(&self.inner.serving, &mut state, &next)?;
            state.catalog = next;
        }
        Ok(())
    }

    pub fn mark_branch_retired(&self, branch_name: &str) -> Result<(), HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)?;
        let mut next = state.catalog.clone();
        next.pending_retire_branches
            .retain(|name| name != branch_name);
        next.inflight_retain_branches
            .retain(|name| name != branch_name);
        if next != state.catalog {
            publish_catalog(&self.inner.serving, &mut state, &next)?;
            state.catalog = next;
        }
        Ok(())
    }

    pub fn settle_drained_pins(&self) -> Result<usize, HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)? {
            self.clear_cached_memberships();
        }
        let pins = self.inner.serving.root_pins().map_err(|_| corrupt())?;
        let settled = pins.settle_drained(pin_write()?).map_err(|_| corrupt())?;
        let before = state.catalog.pending_retire_pins.len();
        state.catalog.pending_retire_pins.retain(|ord| {
            matches!(
                pins.state(RootPinId::from_ordinal(*ord)),
                Some(RootPinState::Active | RootPinState::Expiring)
            )
        });
        if state.catalog.pending_retire_pins.len() != before {
            let next = state.catalog.clone();
            publish_catalog(&self.inner.serving, &mut state, &next)?;
        }
        Ok(settled)
    }

    pub fn basis_locator_summary(&self, commit_id: CommitId) -> Option<(u64, Vec<ChunkMemberId>)> {
        let entry = {
            let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
            state.catalog.bases.get(&commit_id).cloned()?
        };
        let pins = self.inner.serving.root_pins().ok()?;
        let loan = pins.loan(RootPinId::from_ordinal(entry.pin_ordinal)).ok()?;
        let reader = DurableMemberReader::new(
            loan.records().ok()?,
            Arc::clone(&self.inner.corrupted_overrides),
        );
        let chunks = load_basis_manifest_on_demand(&self.inner.residency, &reader, &entry).ok()?;
        Some((
            entry.consolidation_epoch,
            chunks.iter().map(|c| c.chunk_member).collect(),
        ))
    }

    pub fn retained_branch_name(&self, commit_id: CommitId) -> Option<String> {
        let state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        state
            .catalog
            .bases
            .get(&commit_id)
            .and_then(|entry| entry.retained_branch_name.clone())
    }

    pub fn inject_corrupt_member_bytes(&self, id: ChunkMemberId, corrupt_bytes: Vec<u8>) {
        self.inner
            .corrupted_overrides
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, corrupt_bytes);
    }

    pub(super) fn evict_cached_membership(&self, _commit_id: CommitId, _pin_ordinal: u64) {}

    /// No-op retained for Home shutdown cleanup: direct pinned-root member reads
    /// hold no cached `PhysicalRecordReader` guards in `DurableStoreInner`.
    pub(crate) fn clear_cached_memberships(&self) {}

    #[cfg(feature = "certification-test-authority")]
    pub fn set_certification_mutant(&self, mutant: HistoricalCursorCertificationMutant) {
        *self.inner.mutant.lock().unwrap_or_else(|e| e.into_inner()) = mutant;
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn arm_crash_cut(&self, cut: Option<DurableHistoricalCrashCut>) {
        *self
            .inner
            .crash_cut
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = cut;
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn take_crash_cut(&self, target: DurableHistoricalCrashCut) -> bool {
        let mut guard = self
            .inner
            .crash_cut
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if *guard == Some(target) {
            *guard = None;
            true
        } else {
            false
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn active_mutant(&self) -> HistoricalCursorCertificationMutant {
        *self.inner.mutant.lock().unwrap_or_else(|e| e.into_inner())
    }
}
