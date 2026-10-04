//! Checkpoint attestation, capacity and selected-ledger commit for one
//! terminal head retired. A terminal flag is not attestation: the head must
//! be identical in the last namespace-durable checkpoint roster and in the
//! effective roster, so no selected WAL transition still depends on it.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedTerminalReleaseHeadRetirementV1,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, RootPublicationCell,
};

use super::backing::ReleasePublicationAllocationOwner;
use super::heads::SelectedReleaseHeadStep;
use super::{
    PendingReleaseEvent, ReleaseCertificateCapacityDenial, ReleaseHeadCapacityCharge,
    SelectedReleaseCustodyLedger,
};
use crate::physical_runtime::{
    durability::{PhysicalCurrentRootOwner, PhysicalReclaimAttempt},
    CompletedPhysicalMutation, PhysicalRecoveryAllocationAdmission,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum TerminalHeadAttestationDenial {
    SelectedLedgerUnavailable,
    SelectedHeadRootMismatch,
    NoHead,
    NonterminalHead,
    NotCheckpointAttested,
}

/// Sealed: only the selected release ledger, read under the publication-state
/// lock, can state that this exact terminal head is checkpoint-attested at
/// this root.
pub(in crate::physical_runtime) struct CheckpointAttestedTerminalHead {
    root: RootPublicationCell,
    entry: ReleaseCustodyHeadEntryV1,
}

impl CheckpointAttestedTerminalHead {
    pub(in crate::physical_runtime) fn key(&self) -> ReleaseCustodyHeadKeyV1 {
        self.entry.key()
    }

    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.root
    }

    pub(in crate::physical_runtime) const fn entry(&self) -> ReleaseCustodyHeadEntryV1 {
        self.entry
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) const fn fixture(
        root: RootPublicationCell,
        entry: ReleaseCustodyHeadEntryV1,
    ) -> Self {
        Self { root, entry }
    }
}

impl SelectedReleaseCustodyLedger {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn attest_terminal_head(
        &self,
        current: &DurablePhysicalRootManifest,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<CheckpointAttestedTerminalHead, TerminalHeadAttestationDenial> {
        use TerminalHeadAttestationDenial as Denial;
        if self.effective_heads.root() != current.release_custody_head_root() {
            return Err(Denial::SelectedHeadRootMismatch);
        }
        let entry = self.effective_heads.head(key).ok_or(Denial::NoHead)?;
        if !entry.terminal() {
            return Err(Denial::NonterminalHead);
        }
        if self.checkpoint_heads.head(key) != Some(entry) {
            return Err(Denial::NotCheckpointAttested);
        }
        Ok(CheckpointAttestedTerminalHead {
            root: current.root_cell(),
            entry,
        })
    }

    /// The released-drop admission's own backing: the pending-event slot,
    /// the publication closure and the standing checkpoint reservation. The
    /// retirement fence owns no heap, and the transient root frame is
    /// released at once.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn fund_terminal_head_retirement(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        key: ReleaseCustodyHeadKeyV1,
        charge: ReleaseHeadCapacityCharge,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let closure_bytes = self.head_closure_bytes(key, charge)?;
        self.admit_drop_backing(owner, ceiling, closure_bytes, 0, key)
            .map(drop)
    }
}

/// The roster step of a completed member, issued only when the member is the
/// exact step that published `current`. The roster transition checks nothing
/// but endpoint keys, so this join alone binds the ledger's head root to the
/// published root.
fn published_head_step(
    retirement: &PersistedTerminalReleaseHeadRetirementV1,
    completed_root_generation: u64,
    current: &DurablePhysicalRootManifest,
) -> Result<SelectedReleaseHeadStep, ReleaseCertificateCapacityDenial> {
    if retirement.tree_identity() != current.tree_identity()
        || retirement.result_root() != current.release_custody_head_root()
        || completed_root_generation != current.generation()
    {
        return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
    }
    Ok(SelectedReleaseHeadStep::new(
        Some(retirement.source_root()),
        retirement.result_root(),
        retirement.mutation(),
    ))
}

impl PhysicalCurrentRootOwner {
    /// Joins the completed tag-9 member to the exact published fence, then
    /// makes the retirement a pending selected event so the next checkpoint
    /// ratchets the roster. Success releases the fence.
    pub(in crate::physical_runtime) fn commit_terminal_head_retirement(
        &self,
        attempt: &PhysicalReclaimAttempt,
        completed: &CompletedPhysicalMutation,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        use ReleaseCertificateCapacityDenial as Denial;
        let mut guard = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        let state = &mut *guard;
        let current = &state.current_root;
        if !fence.as_ref().is_some_and(|active| {
            active.matches_attempt(attempt.bytes())
                && active.is_published_terminal_head_retirement(completed.mutation_identity())
                && active.selected_root_cell() == current.root_cell()
        }) {
            return Err(Denial::ReclaimFenceMismatch);
        }
        let retirement = completed
            .selected_terminal_head_retirement()
            .ok_or(Denial::SelectedFactMismatch)?;
        let head_step = published_head_step(
            retirement,
            completed.completed_breadth().current_root_generation(),
            current,
        )?;
        let ledger = state
            .release_ledger
            .selected_mut()
            .ok_or(Denial::SelectedLedgerUnavailable)?;
        let event = PendingReleaseEvent::for_retirement(head_step)?;
        let transition = head_step.prepare(&ledger.effective_heads)?;
        if ledger.pending_events.len() == ledger.pending_events.capacity() {
            return Err(Denial::CapacityExhausted);
        }
        // Every join and backing check is complete. Nothing below can fail.
        transition.apply(&mut ledger.effective_heads);
        ledger.pending_events.push(event);
        *fence = None;
        Ok(())
    }
}

#[cfg(test)]
#[path = "terminal_head_retirement/tests.rs"]
mod tests;
