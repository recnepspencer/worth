//! Selected positive no-release marker; never inferred from an empty roster.

#[cfg(feature = "recovery-runtime-owner")]
use crate::physical_runtime::record_serving::RecoveredNoReleaseCustody;
use worth_store_physical_format::ReleaseCheckpointNoReleaseV1;

use super::{
    ReleaseLedgerState, SelectedNoReleaseMarkerBasis, SelectedReleaseCustodyLedger,
    CERTIFICATE_FRAME_OVERHEAD,
};

impl ReleaseLedgerState {
    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_verified_no_release(
        verified: &RecoveredNoReleaseCustody,
    ) -> Self {
        match verified.selected() {
            Some(claim) => {
                Self::from_selected_no_release(claim.marker(), claim.marker_payload_sha256())
            }
            // Before the first checkpoint no release was ever selected: the
            // verified generation-zero basis is the trusted genesis lineage,
            // exactly as a clean reopen of the same media admits it.
            None => Self::Selected(SelectedReleaseCustodyLedger::trusted_genesis()),
        }
    }

    /// The one selected-marker ledger, whether C8 or clean reopen verified it.
    pub(super) fn from_selected_no_release(
        marker: ReleaseCheckpointNoReleaseV1,
        marker_payload_sha256: [u8; 32],
    ) -> Self {
        let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
        ledger.no_release_marker = Some(SelectedNoReleaseMarkerBasis::Selected {
            checkpoint: marker.checkpoint(),
            root_sha256: marker.root_sha256(),
            marker_payload_sha256,
        });
        ledger.used_records = 1;
        ledger.used_bytes = u32::try_from(
            worth_store_physical_format::RELEASE_CHECKPOINT_NO_RELEASE_WIRE_BYTES as u64
                + CERTIFICATE_FRAME_OVERHEAD,
        )
        .expect("fixed no-release certificate fits bounded bytes");
        Self::Selected(ledger)
    }
}
