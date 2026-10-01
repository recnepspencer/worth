//! Selected V2 checkpoint-source roster is installed only after the Store's
//! independent media walk agrees with C8's authenticated checkpoint claim.

pub(super) mod encoding;

use worth_store_physical_format::{
    ReleaseCheckpointAccumulatorV2, ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
    RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES,
};

use super::{
    heads::SelectedReleaseHeadRoster, ReleaseCertificateCapacityDenial, ReleaseLedgerState,
    SelectedReleaseCustodyLedger, CERTIFICATE_FRAME_OVERHEAD,
};
use crate::physical_runtime::recovery_residency::StoreRejoinResidentLedger;
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) enum RecoveredReleaseLedgerDenial {
    SelectedFactMismatch,
    Resident(PhysicalRecoveryRejoinResidentDenial),
}

impl From<ReleaseCertificateCapacityDenial> for RecoveredReleaseLedgerDenial {
    fn from(denial: ReleaseCertificateCapacityDenial) -> Self {
        match denial {
            ReleaseCertificateCapacityDenial::Resident(cause) => Self::Resident(cause),
            _ => Self::SelectedFactMismatch,
        }
    }
}

impl From<PhysicalRecoveryRejoinResidentDenial> for RecoveredReleaseLedgerDenial {
    fn from(value: PhysicalRecoveryRejoinResidentDenial) -> Self {
        Self::Resident(value)
    }
}

impl ReleaseLedgerState {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_verified_v2_source(
        accumulator_v2: ReleaseCheckpointAccumulatorV2,
        source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        source_entries: impl IntoIterator<Item = ReleaseCustodyHeadEntryV1>,
        resident: &mut StoreRejoinResidentLedger,
    ) -> Result<Self, RecoveredReleaseLedgerDenial> {
        let source_heads = SelectedReleaseHeadRoster::from_selected_admitted(
            source_root,
            source_entries,
            resident,
        )?;
        let (head_count, head_digest) = source_heads.commitment()?;
        if head_count != accumulator_v2.head_count()
            || head_digest != accumulator_v2.head_roster_digest()
        {
            return Err(RecoveredReleaseLedgerDenial::SelectedFactMismatch);
        }
        let accumulator = accumulator_v2.base();
        let tip = accumulator.tip();
        let effective_heads = source_heads.clone_admitted(resident)?;
        Ok(Self::Selected(SelectedReleaseCustodyLedger {
            no_release_marker: None,
            used_records: 1,
            used_bytes: u32::try_from(
                RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES as u64 + CERTIFICATE_FRAME_OVERHEAD,
            )
            .map_err(|_| RecoveredReleaseLedgerDenial::SelectedFactMismatch)?,
            cumulative_dropped: accumulator.cumulative_dropped(),
            cumulative_digest: accumulator.cumulative_digest(),
            selected_tip: Some(tip),
            checkpoint: Some(accumulator.checkpoint()),
            pending_events: Vec::new(),
            checkpoint_heads: source_heads,
            effective_heads,
            prior_head_count: head_count,
            prior_head_roster_digest: head_digest,
            terminal: accumulator.terminal(),
            prior_checkpoint_root_sha256: accumulator.root_sha256(),
            prior_accumulator_digest: encoding::accumulator_digest_with_resident(
                accumulator_v2,
                resident,
            )?,
            prior_cumulative_dropped: accumulator.cumulative_dropped(),
            prior_cumulative_digest: accumulator.cumulative_digest(),
            prior_tip: Some(tip),
            prior_terminal: accumulator.terminal(),
        }))
    }
}
