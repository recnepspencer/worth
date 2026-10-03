//! Selected positive no-release marker; never inferred from an empty roster.

use worth_store_physical_format::ReleaseCheckpointNoReleaseV1;
use worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody;

use super::{
    ReleaseLedgerState, SelectedNoReleaseMarkerBasis, SelectedReleaseCustodyLedger,
    CERTIFICATE_FRAME_OVERHEAD,
};

impl ReleaseLedgerState {
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn from_verified_no_release(
        verified: &VerifiedSelectedNoReleaseCustody,
    ) -> Self {
        Self::from_selected_no_release(verified.marker(), verified.marker_payload_sha256())
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
