//! Release certificate capacity is selected-custody state, never an inferred
//! empty counter on reopen. The pre-effect lease cutover waits for C8's sealed
//! checkpoint-plus-tail ledger and the typed tag7 encoded-size contract.

mod charge;
mod checkpoint_commit;
mod commit;
mod heads;
mod no_release;
mod pending_wal;
mod reopen;
pub(in crate::physical_runtime) use reopen::RecoveredReleaseLedgerDenial;
mod snapshot;

use std::sync::{Arc, Mutex};

use worth_store_physical_format::{
    OriginalDropReservationRequestV1, PersistedRecordIdentity, PhysicalCheckpointIdentity,
    ReleaseCustodyHeadKeyV1, ReleasedDropPredecessorV1, ReleasedDropTipProvenanceV1,
    ReleasedDropWalFateWitnessV1, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS, TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES,
};

use super::certificate_capacity::CheckpointCustodyOrigin;
use super::reclaim::{PhysicalReclaimAttempt, ReclaimFenceState};
use super::PhysicalCurrentRootOwner;
pub(in crate::physical_runtime) use charge::ReleaseHeadCapacityCharge;
pub(in crate::physical_runtime) use heads::SelectedReleaseHeadBasis;
pub use heads::SelectedReleaseHeadDenial;
use heads::{SelectedReleaseHeadRoster, SelectedReleaseHeadStep};

const CERTIFICATE_FRAME_OVERHEAD: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum ReleaseCertificateCapacityDenial {
    SelectedLedgerUnavailable,
    ReclaimFenceMismatch,
    CapacityExhausted,
    SelectedFactMismatch,
}

#[derive(Clone, Copy)]
pub(super) struct SelectedReleaseBatchBasis {
    pub(super) head_step: Option<SelectedReleaseHeadStep>,
    pub(super) descriptor_record: PersistedRecordIdentity,
    pub(super) descriptor_frame_sha256: [u8; 32],
    pub(super) custody_digest: [u8; 32],
    pub(super) reservation_record: PersistedRecordIdentity,
    pub(super) reservation_frame_sha256: [u8; 32],
    pub(super) request: OriginalDropReservationRequestV1,
    pub(super) fate: ReleasedDropWalFateWitnessV1,
    pub(super) candidate_root_generation: u64,
    pub(super) candidate_root_sha256: [u8; 32],
    pub(super) predecessor: Option<ReleasedDropPredecessorV1>,
    pub(super) cumulative_dropped: u64,
    pub(super) cumulative_digest: [u8; 32],
    pub(super) terminal: bool,
}

pub(super) struct ReleaseCertificatePending {
    pub(super) needed_records: u16,
    pub(super) worst_case_encoded_bytes: u32,
    pub(super) effect_may_exist: bool,
}

/// Positive lineage for a Store that has never selected a release. Absence of
/// tag-7 records on reopened media is never converted into this state.
#[derive(Clone, Copy)]
pub(super) enum SelectedNoReleaseMarkerBasis {
    TrustedFreshGenesis,
    Selected {
        checkpoint: PhysicalCheckpointIdentity,
        root_sha256: [u8; 32],
        marker_payload_sha256: [u8; 32],
    },
}

impl SelectedNoReleaseMarkerBasis {
    pub(super) fn prior(self) -> (u64, [u8; 32], [u8; 32]) {
        match self {
            Self::TrustedFreshGenesis => (0, [0; 32], [0; 32]),
            Self::Selected {
                checkpoint,
                root_sha256,
                marker_payload_sha256,
            } => (
                checkpoint.sequence().get(),
                root_sha256,
                marker_payload_sha256,
            ),
        }
    }
}

/// One pre-effect release batch reservation against the selected ledger. A
/// dropped uncertain lease leaves the reclaim fence quarantined for reopen.
pub(in crate::physical_runtime) struct ReleaseCertificateCapacityLease {
    fence: Arc<Mutex<Option<ReclaimFenceState>>>,
    attempt: [u8; 16],
}

impl ReleaseCertificateCapacityLease {
    pub(in crate::physical_runtime) fn mark_effect_may_exist(&mut self) -> bool {
        let mut fence = self
            .fence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(fence) = fence
            .as_mut()
            .filter(|fence| fence.matches_attempt(self.attempt))
        else {
            return false;
        };
        let Some(pending) = fence.release_certificate_pending.as_mut() else {
            return false;
        };
        pending.effect_may_exist = true;
        true
    }
}

impl Drop for ReleaseCertificateCapacityLease {
    fn drop(&mut self) {
        let mut fence = self
            .fence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(fence) = fence
            .as_mut()
            .filter(|fence| fence.matches_attempt(self.attempt))
        {
            if fence.is_pre_effect_payload_drop()
                && fence
                    .release_certificate_pending
                    .as_ref()
                    .is_some_and(|pending| !pending.effect_may_exist)
            {
                fence.release_certificate_pending = None;
            }
        }
    }
}

/// C8-reconciled selected checkpoint ratchet plus selected tail. Its fields
/// remain private; only trusted genesis may start empty without a C8 handoff.
pub(in crate::physical_runtime) struct SelectedReleaseCustodyLedger {
    no_release_marker: Option<SelectedNoReleaseMarkerBasis>,
    used_records: u16,
    used_bytes: u32,
    cumulative_dropped: u64,
    cumulative_digest: [u8; 32],
    selected_tip: Option<ReleasedDropTipProvenanceV1>,
    checkpoint: Option<PhysicalCheckpointIdentity>,
    pending_batches: Vec<SelectedReleaseBatchBasis>,
    /// Ordered selected WAL head effects, including terminal retirement
    /// effects that have no tag-7 Batch record.
    pending_head_steps: Vec<SelectedReleaseHeadStep>,
    /// The last namespace-durable checkpoint's source roster.
    checkpoint_heads: SelectedReleaseHeadRoster,
    /// The selected root after all pending WAL-backed head transitions.
    effective_heads: SelectedReleaseHeadRoster,
    prior_head_count: u64,
    prior_head_roster_digest: [u8; 32],
    terminal: bool,
    prior_checkpoint_root_sha256: [u8; 32],
    prior_accumulator_digest: [u8; 32],
    prior_cumulative_dropped: u64,
    prior_cumulative_digest: [u8; 32],
    prior_tip: Option<ReleasedDropTipProvenanceV1>,
    prior_terminal: bool,
}

impl SelectedReleaseCustodyLedger {
    fn trusted_genesis() -> Self {
        Self {
            no_release_marker: Some(SelectedNoReleaseMarkerBasis::TrustedFreshGenesis),
            used_records: 0,
            used_bytes: 0,
            cumulative_dropped: 0,
            cumulative_digest: [0; 32],
            selected_tip: None,
            checkpoint: None,
            pending_batches: Vec::new(),
            pending_head_steps: Vec::new(),
            checkpoint_heads: SelectedReleaseHeadRoster::empty(),
            effective_heads: SelectedReleaseHeadRoster::empty(),
            prior_head_count: 0,
            prior_head_roster_digest: [0; 32],
            terminal: false,
            prior_checkpoint_root_sha256: [0; 32],
            prior_accumulator_digest: [0; 32],
            prior_cumulative_dropped: 0,
            prior_cumulative_digest: [0; 32],
            prior_tip: None,
            prior_terminal: false,
        }
    }

    pub(in crate::physical_runtime) const fn used_certificate_records(&self) -> u16 {
        self.used_records
    }
    pub(in crate::physical_runtime) const fn used_certificate_bytes(&self) -> u32 {
        self.used_bytes
    }
    pub(in crate::physical_runtime) const fn cumulative_dropped(&self) -> u64 {
        self.cumulative_dropped
    }
    pub(in crate::physical_runtime) const fn cumulative_digest(&self) -> [u8; 32] {
        self.cumulative_digest
    }
    pub(in crate::physical_runtime) const fn selected_tip(
        &self,
    ) -> Option<ReleasedDropTipProvenanceV1> {
        self.selected_tip
    }
    pub(in crate::physical_runtime) const fn checkpoint_identity(
        &self,
    ) -> Option<PhysicalCheckpointIdentity> {
        self.checkpoint
    }

    pub(super) fn admits_worst_case(
        &self,
        anchored: bool,
        needed_records: u16,
        worst_case_encoded_bytes: u32,
    ) -> bool {
        // The caller's bound must include both newly selected V3 records and
        // the future tag7 ratchet. It is not yet allowed into live release.
        if needed_records < 2 || worst_case_encoded_bytes == 0 {
            return false;
        }
        let tier_records = u64::from(anchored);
        let tier_bytes = if anchored {
            TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES as u64 + CERTIFICATE_FRAME_OVERHEAD
        } else {
            0
        };
        u64::from(self.used_records)
            .checked_add(u64::from(needed_records))
            .and_then(|count| count.checked_add(tier_records))
            .is_some_and(|count| count <= MAX_CHECKPOINT_CERTIFICATE_RECORDS)
            && u64::from(self.used_bytes)
                .checked_add(u64::from(worst_case_encoded_bytes))
                .and_then(|bytes| bytes.checked_add(tier_bytes))
                .is_some_and(|bytes| bytes <= MAX_CHECKPOINT_CERTIFICATE_BYTES)
    }
}

pub(super) enum ReleaseLedgerState {
    Unavailable,
    Selected(SelectedReleaseCustodyLedger),
}

impl ReleaseLedgerState {
    pub(super) fn from_verified(
        verified: &worth_store_recovery_physics::VerifiedSelectedCheckpointCustody,
    ) -> Self {
        let accumulator = verified.accumulator();
        let tip = accumulator.tip();
        Self::Selected(SelectedReleaseCustodyLedger {
            no_release_marker: None,
            used_records: verified.release_certificate_record_count(),
            used_bytes: verified.release_certificate_encoded_bytes(),
            cumulative_dropped: accumulator.cumulative_dropped(),
            cumulative_digest: accumulator.cumulative_digest(),
            selected_tip: Some(tip),
            checkpoint: Some(accumulator.checkpoint()),
            pending_batches: Vec::new(),
            pending_head_steps: Vec::new(),
            checkpoint_heads: SelectedReleaseHeadRoster::empty(),
            effective_heads: SelectedReleaseHeadRoster::empty(),
            prior_head_count: 0,
            prior_head_roster_digest: [0; 32],
            terminal: accumulator.terminal(),
            prior_checkpoint_root_sha256: accumulator.root_sha256(),
            prior_accumulator_digest: verified.accumulator_payload_sha256(),
            prior_cumulative_dropped: accumulator.cumulative_dropped(),
            prior_cumulative_digest: accumulator.cumulative_digest(),
            prior_tip: Some(tip),
            prior_terminal: accumulator.terminal(),
        })
    }

    pub(super) fn from_origin(origin: CheckpointCustodyOrigin) -> Self {
        match origin {
            CheckpointCustodyOrigin::FreshGenesis => {
                Self::Selected(SelectedReleaseCustodyLedger::trusted_genesis())
            }
            CheckpointCustodyOrigin::ReopenRequiresC8 => Self::Unavailable,
        }
    }

    pub(super) fn selected(&self) -> Option<&SelectedReleaseCustodyLedger> {
        match self {
            Self::Unavailable => None,
            Self::Selected(ledger) => Some(ledger),
        }
    }

    pub(super) fn selected_mut(&mut self) -> Option<&mut SelectedReleaseCustodyLedger> {
        match self {
            Self::Unavailable => None,
            Self::Selected(ledger) => Some(ledger),
        }
    }
}

impl PhysicalCurrentRootOwner {
    /// Released-generation custody needs a selected tag7 basis before any
    /// subsequent checkpoint. Mark it while the admitted reclaim attempt is
    /// still pre-effect; no ordinary legacy capture may bypass this handoff.
    pub(in crate::physical_runtime) fn require_release_certificate_for_attempt(
        &self,
        attempt: &PhysicalReclaimAttempt,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        let mut state = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if !fence.as_ref().is_some_and(|active| {
            active.matches_attempt(attempt.bytes()) && active.is_pre_effect_payload_drop()
        }) {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        if state.release_ledger.selected().is_none() {
            return Err(ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable);
        }
        if !state
            .checkpoint_custody
            .require_release_certificate(attempt.bytes())
        {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        Ok(())
    }

    /// Dormant until C8 installs a selected reopen ledger and Store derives
    /// `worst_case_encoded_bytes` from typed V3/tag7 records. No release path
    /// calls this method during the transitional format cutover.
    pub(in crate::physical_runtime) fn reserve_release_certificate_capacity(
        &self,
        attempt: &PhysicalReclaimAttempt,
        key: ReleaseCustodyHeadKeyV1,
        needed_records: u16,
        worst_case_encoded_bytes: u32,
        head_charge: ReleaseHeadCapacityCharge,
    ) -> Result<ReleaseCertificateCapacityLease, ReleaseCertificateCapacityDenial> {
        let state = self.lock_publication_state();
        let mut fence = self.lock_reclaim();
        let active = fence
            .as_mut()
            .filter(|active| active.matches_attempt(attempt.bytes()))
            .ok_or(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch)?;
        if !active.is_pre_effect_payload_drop() || active.release_certificate_pending.is_some() {
            return Err(ReleaseCertificateCapacityDenial::ReclaimFenceMismatch);
        }
        let ledger = state
            .release_ledger
            .selected()
            .ok_or(ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable)?;
        if !ledger.admits_worst_case(
            state.current_root.tier_epoch_anchor().is_some(),
            needed_records,
            worst_case_encoded_bytes,
        ) {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        if ledger.effective_heads.root() != state.current_root.release_custody_head_root() {
            return Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
        }
        if !ledger.admits_head_closure(key, head_charge, self.recovery_allocation.byte_limit())? {
            return Err(ReleaseCertificateCapacityDenial::CapacityExhausted);
        }
        active.release_certificate_pending = Some(ReleaseCertificatePending {
            needed_records,
            worst_case_encoded_bytes,
            effect_may_exist: false,
        });
        Ok(ReleaseCertificateCapacityLease {
            fence: attempt.certificate_fence(),
            attempt: attempt.bytes(),
        })
    }
}

#[cfg(test)]
mod tests;
