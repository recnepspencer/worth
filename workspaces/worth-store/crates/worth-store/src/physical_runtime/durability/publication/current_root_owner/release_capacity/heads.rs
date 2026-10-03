//! Selected per-object release heads. The checkpoint-source roster and the
//! effective post-WAL roster have different lifetimes and are never compared
//! as though they described the same root.

use worth_store_physical_format::{
    PersistedReleaseCustodyHeadEffectV1, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadRosterDigestV1, ReleasedDropPredecessorV1,
};

mod checkpoint_fold;
mod selection;
#[cfg(test)]
#[path = "heads/tests.rs"]
mod tests;
mod transition;
pub(in crate::physical_runtime) use selection::SelectedReleaseHeadBasis;
pub use selection::SelectedReleaseHeadDenial;

use super::ReleaseCertificateCapacityDenial;
use crate::physical_runtime::durability::publication::current_root_owner::{
    PhysicalCurrentRootOwner, PhysicalReclaimAttempt,
};
use crate::physical_runtime::recovery_residency::StoreRejoinResidentLedger;

use super::reopen::RecoveredReleaseLedgerDenial;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct SelectedReleaseHeadStep {
    source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    mutation: ReleaseCustodyHeadMutationV1,
}

impl SelectedReleaseHeadStep {
    pub(super) fn from_effect(effect: &PersistedReleaseCustodyHeadEffectV1) -> Self {
        Self::new(
            effect.source_root(),
            Some(effect.result_root()),
            effect.mutation(),
        )
    }

    pub(super) fn new(
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        mutation: ReleaseCustodyHeadMutationV1,
    ) -> Self {
        Self {
            source_root,
            result_root,
            mutation,
        }
    }

    pub(super) fn mutation(self) -> ReleaseCustodyHeadMutationV1 {
        self.mutation
    }
}

pub(in crate::physical_runtime::durability::publication::current_root_owner) struct SelectedReleaseHeadRoster
{
    root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    /// Canonical key order is also the checkpoint roster digest order.
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    allocation_custody: Option<std::sync::Arc<super::backing::LiveReleaseAllocation>>,
}

impl SelectedReleaseHeadRoster {
    pub(super) fn reserve_for_key_live(
        &mut self,
        key: ReleaseCustodyHeadKeyV1,
        window: &mut super::backing::LiveBackingWindow<'_>,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        if self.head(key).is_none() {
            window.grow_vec(&mut self.entries, 1)?;
        }
        Ok(())
    }

    pub(super) fn len(&self) -> u64 {
        self.entries.len() as u64
    }

    pub(super) fn empty() -> Self {
        Self {
            root: None,
            entries: Vec::new(),
            allocation_custody: None,
        }
    }

    pub(super) fn root(&self) -> Option<ReleaseCustodyHeadBlockReferenceV1> {
        self.root
    }

    pub(super) fn head(&self, key: ReleaseCustodyHeadKeyV1) -> Option<ReleaseCustodyHeadEntryV1> {
        self.entries
            .binary_search_by_key(&key, |entry| entry.key())
            .ok()
            .map(|index| self.entries[index])
    }

    /// Only the owned flat backing is counted; inline root fields are excluded.
    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        self.entries
            .capacity()
            .checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>())?
            .try_into()
            .ok()
    }

    #[cfg(test)]
    pub(super) fn reserve_for_key(
        &mut self,
        key: ReleaseCustodyHeadKeyV1,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), RecoveredReleaseLedgerDenial> {
        if self.head(key).is_none() {
            resident.grow_vec(&mut self.entries, 1)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn from_selected(
        root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        entries: impl IntoIterator<Item = ReleaseCustodyHeadEntryV1>,
    ) -> Result<Self, ReleaseCertificateCapacityDenial> {
        let mut roster = Self {
            root,
            entries: Vec::new(),
            allocation_custody: None,
        };
        let mut prior_key = None;
        for entry in entries {
            if prior_key.is_some_and(|prior| prior >= entry.key()) {
                return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
            }
            roster.entries.push(entry);
            prior_key = Some(entry.key());
        }
        if roster.root.is_some() != !roster.entries.is_empty() {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        roster.commitment()?;
        Ok(roster)
    }

    /// Recovery construction charges each backing before copying selected
    /// media entries into the Store-owned roster.
    pub(super) fn from_selected_admitted(
        root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        entries: impl IntoIterator<Item = ReleaseCustodyHeadEntryV1>,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, RecoveredReleaseLedgerDenial> {
        let entries = entries.into_iter();
        let (lower, upper) = entries.size_hint();
        let mut selected = if upper == Some(lower) {
            resident.reserve_vec(lower)?
        } else {
            Vec::new()
        };
        let mut prior_key = None;
        for entry in entries {
            if prior_key.is_some_and(|prior| prior >= entry.key()) {
                return Err(RecoveredReleaseLedgerDenial::SelectedFactMismatch);
            }
            resident.grow_vec(&mut selected, 1)?;
            selected.push(entry);
            prior_key = Some(entry.key());
        }
        let roster = Self {
            root,
            entries: selected,
            allocation_custody: None,
        };
        roster
            .commitment()
            .map_err(RecoveredReleaseLedgerDenial::from)?;
        Ok(roster)
    }

    pub(super) fn clone_admitted(
        &self,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, RecoveredReleaseLedgerDenial> {
        let mut entries = resident.reserve_vec(self.entries.len())?;
        entries.extend_from_slice(&self.entries);
        Ok(Self {
            root: self.root,
            entries,
            allocation_custody: None,
        })
    }

    /// Compare C8's final typed roster without a second retained Store copy.
    pub(super) fn matches_selected(
        &self,
        root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        entries: &[ReleaseCustodyHeadEntryV1],
        digest: [u8; 32],
    ) -> Result<bool, ReleaseCertificateCapacityDenial> {
        if root.is_some() != !entries.is_empty()
            || root.is_some_and(|reference| {
                entries.first().map(|entry| entry.key()) != Some(reference.first())
                    || entries.last().map(|entry| entry.key()) != Some(reference.last())
            })
        {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        let mut observed = ReleaseCustodyHeadRosterDigestV1::new(root, u64::MAX);
        for entry in entries.iter().copied() {
            observed
                .push(entry)
                .map_err(|_| ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        }
        let (count, observed_digest) = observed.finish();
        Ok(self.root == root
            && self.entries.as_slice() == entries
            && self.commitment()? == (count, observed_digest)
            && observed_digest == digest)
    }

    pub(super) fn commitment(&self) -> Result<(u64, [u8; 32]), ReleaseCertificateCapacityDenial> {
        if self.root.is_some() != !self.entries.is_empty() {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        if self.root.is_some_and(|root| {
            self.entries.first().map(|entry| entry.key()) != Some(root.first())
                || self.entries.last().map(|entry| entry.key()) != Some(root.last())
        }) {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        let mut digest = ReleaseCustodyHeadRosterDigestV1::new(self.root, u64::MAX);
        for entry in self.entries.iter().copied() {
            digest
                .push(entry)
                .map_err(|_| ReleaseCertificateCapacityDenial::SelectedFactMismatch)?;
        }
        Ok(digest.finish())
    }

    #[cfg(test)]
    pub(super) fn apply_transition(
        &mut self,
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        mutation: ReleaseCustodyHeadMutationV1,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let transition =
            SelectedReleaseHeadStep::new(source_root, result_root, mutation).prepare(self)?;
        if transition.inserts() {
            self.entries
                .try_reserve_exact(1)
                .map_err(|_| ReleaseCertificateCapacityDenial::CapacityExhausted)?;
        }
        transition.apply(self);
        Ok(())
    }

    pub(super) fn apply_transition_admitted(
        &mut self,
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        result_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        mutation: ReleaseCustodyHeadMutationV1,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<(), RecoveredReleaseLedgerDenial> {
        let transition =
            SelectedReleaseHeadStep::new(source_root, result_root, mutation).prepare(self)?;
        if transition.inserts() {
            resident.grow_vec(&mut self.entries, 1)?;
        }
        transition.apply(self);
        Ok(())
    }
}

fn valid_successor(
    prior: Option<ReleaseCustodyHeadEntryV1>,
    next: ReleaseCustodyHeadEntryV1,
) -> bool {
    match prior {
        None => next.predecessor().is_none(),
        Some(prior) => {
            !prior.terminal()
                && next.key() == prior.key()
                && next.source_basis_digest() == prior.source_basis_digest()
                && next.cumulative_dropped() > prior.cumulative_dropped()
                && next.predecessor()
                    == ReleasedDropPredecessorV1::new(
                        prior.descriptor_record(),
                        prior.descriptor_frame_sha256(),
                    )
                    .ok()
        }
    }
}

impl PhysicalCurrentRootOwner {
    /// Reads the exact selected source head while the reservation-published
    /// payload-drop fence still pins the root. A global release tip is never
    /// consulted for per-object continuation.
    pub(in crate::physical_runtime) fn selected_release_head_for_attempt(
        &self,
        attempt: &PhysicalReclaimAttempt,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<Option<ReleaseCustodyHeadEntryV1>, ReleaseCertificateCapacityDenial> {
        let state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if !fence.as_ref().is_some_and(|active| {
            active.matches_attempt(attempt.bytes())
                && active.is_reservation_published_payload_drop()
                && active.expected_root == state.current_root.root_cell()
        }) {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        let ledger = state
            .release_ledger
            .selected()
            .ok_or(ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable)?;
        if ledger.effective_heads.root() != state.current_root.release_custody_head_root() {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        Ok(ledger.effective_heads.head(key))
    }
}
