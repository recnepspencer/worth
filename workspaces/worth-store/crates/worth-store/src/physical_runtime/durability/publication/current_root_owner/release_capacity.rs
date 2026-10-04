//! Release certificate capacity is selected-custody state, never an inferred
//! empty counter on reopen. The pre-effect lease cutover waits for C8's sealed
//! checkpoint-plus-tail ledger and the typed tag7 encoded-size contract.

mod admission;
pub(super) mod backing;
pub(super) mod capture_envelope;
#[cfg(feature = "certification-test-authority")]
mod certification_observation;
#[cfg(feature = "certification-test-authority")]
pub use certification_observation::CertificationReleaseHeadObservation;
mod charge;
mod checkpoint_commit;
mod commit;
mod event;
mod heads;
mod no_release;
#[cfg(feature = "recovery-runtime-owner")]
mod pending_wal;
#[cfg(any(test, feature = "recovery-runtime-owner"))]
mod reopen;
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use reopen::RecoveredReleaseLedgerDenial;
mod snapshot;
mod terminal_head_retirement;
pub(in crate::physical_runtime) use terminal_head_retirement::{
    CheckpointAttestedTerminalHead, TerminalHeadAttestationDenial,
};

use std::sync::{Arc, Mutex};

use worth_store_physical_format::{
    OriginalDropReservationRequestV1, PersistedRecordIdentity, PhysicalCheckpointIdentity,
    ReleaseCustodyHeadKeyV1, ReleasedDropPredecessorV1, ReleasedDropTipProvenanceV1,
    ReleasedDropWalFateWitnessV1, MAX_CHECKPOINT_CERTIFICATE_BYTES,
    MAX_CHECKPOINT_CERTIFICATE_RECORDS, TIER_EPOCH_CHECKPOINT_CERTIFICATE_WIRE_BYTES,
};

use super::certificate_capacity::CheckpointCustodyOrigin;
use super::reclaim::{PhysicalReclaimAttempt, ReclaimFenceState};

pub(in crate::physical_runtime) use charge::ReleaseHeadCapacityCharge;
use event::PendingReleaseEvent;
pub(in crate::physical_runtime) use heads::SelectedReleaseHeadBasis;
pub use heads::SelectedReleaseHeadDenial;
use heads::SelectedReleaseHeadRoster;

const CERTIFICATE_FRAME_OVERHEAD: u64 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseCertificateCapacityDenial {
    SelectedLedgerUnavailable,
    ReclaimFenceMismatch,
    CapacityExhausted,
    SelectedFactMismatch,
    Resident(crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial),
}

#[derive(Clone, Copy)]
pub(super) struct SelectedReleaseBatchBasis {
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
    pub(super) key: ReleaseCustodyHeadKeyV1,
    pub(super) root_frame: backing::FundedRootFrame,
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
    /// One ordered selected WAL stream. A drop owns its exact head transition;
    /// a terminal retirement has no tag-7 Batch certificate.
    pending_events: Vec<PendingReleaseEvent>,
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
    // Declared after owned vectors: their backing dies before its grant.
    allocation_custody: Option<Arc<backing::LiveReleaseAllocation>>,
    /// The standing checkpoint reservation: the committed checkpoint roster
    /// plus the next capture's whole envelope. Drop admission grows it before
    /// any effect; a capture consumes it, so checkpointing never funds fresh.
    capture_custody: Option<Arc<backing::LiveReleaseAllocation>>,
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
            pending_events: Vec::new(),
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
            allocation_custody: None,
            capture_custody: None,
        }
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
    pub(super) fn from_origin(origin: CheckpointCustodyOrigin) -> Self {
        match origin {
            CheckpointCustodyOrigin::FreshGenesis => {
                Self::Selected(SelectedReleaseCustodyLedger::trusted_genesis())
            }
            CheckpointCustodyOrigin::CleanReopen(
                super::clean_reopen::CleanReopenCheckpointCustody::TrustedGenesis { .. },
            ) => Self::Selected(SelectedReleaseCustodyLedger::trusted_genesis()),
            CheckpointCustodyOrigin::CleanReopen(
                super::clean_reopen::CleanReopenCheckpointCustody::SelectedNoRelease {
                    marker,
                    marker_payload_sha256,
                    ..
                },
            ) => Self::from_selected_no_release(marker, marker_payload_sha256),
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

#[cfg(test)]
mod tests;
