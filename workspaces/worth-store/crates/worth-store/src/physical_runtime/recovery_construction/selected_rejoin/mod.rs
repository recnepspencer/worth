//! Rejoins a C.8 custody claim against one Store-admitted physical media
//! session before a recovered serving capability can be issued.

pub(super) mod control_frames;
pub(super) mod head_v2;
pub(super) mod no_release;
pub(super) mod pending_wal_release;
pub(super) mod release_heads;
pub(super) mod resident;
pub(in crate::physical_runtime) use resident::PhysicalRecoveryRejoinResidentDenial;
pub(super) mod root_checkpoint;
pub(super) mod tier;
pub(super) mod wal_fate;
mod wal_inventory;
pub(in crate::physical_runtime) use control_frames::SelectedControlMediaFingerprint;
pub(in crate::physical_runtime) use wal_inventory::SelectedWalMediaFingerprint;

use worth_store_physical_backend::RecoveryFilesystemQualificationError;

const MAX_CHECKPOINT_BYTES: u64 = 256 << 20;
const MAX_CONTROL_FRAME_BYTES: u64 = 64 << 20;
const MAX_DISCOVERY_BYTES: u64 = 1 << 30;
const MAX_DISCOVERY_ENTRIES: u64 = 65_536;
const MAX_CLEANUP_SAMPLE_BYTES: u64 = 128 << 20;

#[derive(Debug)]
pub(super) enum SelectedMediaRejoinDenial {
    Resident(PhysicalRecoveryRejoinResidentDenial),
    ResidentBoundary {
        boundary: crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    WalResident {
        boundary: Option<crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary>,
        stage: crate::physical_runtime::PhysicalRecoveryWalResidentStage,
        artifact_count: usize,
        artifact_ordinal: Option<usize>,
        frame_offset: Option<u64>,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    ResidentRead {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: PhysicalRecoveryRejoinResidentDenial,
    },
    ReadBufferLengthMismatch {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        observed: usize,
    },
    Qualification(RecoveryFilesystemQualificationError),
    Discovery(crate::physical_runtime::RecoveryDiscoveryFailure),
    MissingSelector,
    MissingRoot,
    MissingCheckpoint,
    MissingRoute,
    MissingFrame,
    RootBinding,
    CheckpointBinding,
    CertificateRoster,
    RoutingFrame,
    UnsupportedSelectedPlacement,
    ControlFrame,
    BoundExceeded,
    WalFate,
}

impl SelectedMediaRejoinDenial {
    pub(super) fn at_resident_boundary(
        self,
        boundary: crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary,
    ) -> Self {
        match self {
            Self::Resident(cause) => Self::ResidentBoundary { boundary, cause },
            Self::WalResident {
                boundary: existing,
                stage,
                artifact_count,
                artifact_ordinal,
                frame_offset,
                cause,
            } => Self::WalResident {
                boundary: existing.or(Some(boundary)),
                stage,
                artifact_count,
                artifact_ordinal,
                frame_offset,
                cause,
            },
            other => other,
        }
    }
}
