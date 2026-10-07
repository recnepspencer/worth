//! Checkpoint custody that ordinary reopen verifies itself (the clean case):
//! the selected certificate binds the loaded root and checkpoint source, and
//! the retained WAL suffix has no released-drop member. Any retained released
//! drop keeps custody unavailable until the C.8 handoff.

use worth_store_physical_format::{ReleaseCheckpointNoReleaseV1, TierEpochCheckpointCertificateV1};

use super::certificate_capacity::CheckpointCustodyOrigin;
use crate::physical_runtime::durability::RetainedWalReleaseEvidence;

/// Custody proven by Store from the selected media, without C.8.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CleanReopenCheckpointCustody {
    /// No checkpoint was ever selected and the whole WAL history is retained.
    /// `first_root`: the loaded root is the Store's first, the only root
    /// whose WAL history may be empty.
    TrustedGenesis { first_root: bool },
    /// The selected checkpoint's positive NoRelease marker, plus its leading
    /// tier certificate when the loaded root still carries that tier anchor.
    SelectedNoRelease {
        marker: ReleaseCheckpointNoReleaseV1,
        marker_payload_sha256: [u8; 32],
        tier: Option<TierEpochCheckpointCertificateV1>,
    },
}

/// Checkpoint custody as open knows it before durability reopen has admitted
/// the retained WAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CheckpointCustodyCandidate {
    FreshGenesis,
    ReopenRequiresC8,
    CleanReopen(CleanReopenCheckpointCustody),
}

impl CheckpointCustodyCandidate {
    /// Joins the root-verified candidate to the admitted retained WAL.
    pub(in crate::physical_runtime) const fn admit(
        self,
        wal: RetainedWalReleaseEvidence,
    ) -> CheckpointCustodyOrigin {
        match self {
            Self::FreshGenesis => CheckpointCustodyOrigin::FreshGenesis,
            Self::CleanReopen(
                custody @ CleanReopenCheckpointCustody::TrustedGenesis { first_root },
            ) if wal.admits_trusted_genesis_custody(first_root) => {
                CheckpointCustodyOrigin::CleanReopen(custody)
            }
            Self::CleanReopen(custody @ CleanReopenCheckpointCustody::SelectedNoRelease { .. })
                if wal.admits_selected_checkpoint_custody() =>
            {
                CheckpointCustodyOrigin::CleanReopen(custody)
            }
            Self::ReopenRequiresC8 | Self::CleanReopen(_) => {
                CheckpointCustodyOrigin::ReopenRequiresC8
            }
        }
    }

    /// A tier-anchored root opens only through its verified tier custody, so
    /// a retained WAL that refuses this candidate requires C.8.
    pub(in crate::physical_runtime) const fn requires_clean_custody(self) -> bool {
        matches!(
            self,
            Self::CleanReopen(CleanReopenCheckpointCustody::SelectedNoRelease {
                tier: Some(_),
                ..
            })
        )
    }
}
