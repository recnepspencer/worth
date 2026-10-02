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
pub(in crate::physical_runtime) enum SelectedMediaRejoinDenial {
    Resident(PhysicalRecoveryRejoinResidentDenial),
    WalReadOwnership(crate::physical_runtime::PhysicalRecoveryRejoinResidentAdmissionDenial),
    WalRead {
        boundary: Option<crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary>,
        cause: crate::physical_runtime::FundedRecoveryWalReadFailure,
    },
    WalAdmission {
        boundary: Option<crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary>,
        cause: crate::physical_runtime::RecoveryWalAllocationDenial,
    },
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
    RecordReadAllocation {
        artifact: worth_store_physical_backend::RecoveryDiscoveryArtifact,
        offset: u64,
        requested: usize,
        cause: crate::physical_runtime::PhysicalRecoveryObservationAllocationDenial,
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
    BindingSampling {
        boundary: Option<crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary>,
        cause: crate::physical_runtime::StoreRecoveryBindingSampleAllocationDenial,
    },
    WalFate,
}

impl SelectedMediaRejoinDenial {
    pub(super) fn binding_sampling(
        failure: crate::physical_runtime::StoreRecoveryBindingSampleFailure,
    ) -> Self {
        match failure.allocation_denial() {
            Some(cause) => Self::BindingSampling {
                boundary: None,
                cause: cause.clone(),
            },
            None => Self::WalFate,
        }
    }

    pub(super) fn at_resident_boundary(
        self,
        boundary: crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary,
    ) -> Self {
        match self {
            Self::WalRead {
                boundary: existing,
                cause,
            } => Self::WalRead {
                boundary: existing.or(Some(boundary)),
                cause,
            },
            Self::WalAdmission {
                boundary: existing,
                cause,
            } => Self::WalAdmission {
                boundary: existing.or(Some(boundary)),
                cause,
            },
            Self::BindingSampling {
                boundary: existing,
                cause,
            } => Self::BindingSampling {
                boundary: existing.or(Some(boundary)),
                cause,
            },
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
