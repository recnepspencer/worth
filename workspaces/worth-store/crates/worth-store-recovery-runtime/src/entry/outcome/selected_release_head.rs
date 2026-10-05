//! Causal denials for the mandatory checkpoint-source V2 custody gate.
//! These observations never grant selected-media or release authority.

use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_physical_format::{
    BlobRecordDenial, PersistedRecordIdentity, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadDenial,
};
use worth_store_recovery_physics::SelectedCustodyDenial;

use crate::integrity_ingress::RecoveryIntegrityIngressRejection;

use super::PhysicalRecoveryPageAdmissionDenial;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoverySelectedReleaseHeadDenial {
    MissingCheckpoint,
    ObservationByteLimit,
    ManifestEntryLimit,
    /// The checkpoint's verified roster counts more heads than recovery
    /// admits manifest entries.
    RosterEntryLimit {
        observed: u64,
        admitted: u64,
    },
    ResidentBoundExceeded {
        required: u64,
        admitted: u64,
    },
    SourceRootRead {
        generation: u64,
        failure: RecoveryDiscoveryFailure,
    },
    SourceRootIntegrity {
        generation: u64,
        denial: RecoveryIntegrityIngressRejection,
    },
    SourceRootFormatMismatch,
    SourceRoutes(PhysicalRecoveryPageAdmissionDenial),
    WalkLimits,
    Roster(SelectedCustodyDenial),
    HeadWalk(PhysicalRecoveryReleaseHeadWalkDenial),
    Control(PhysicalRecoveryReleaseHeadControlDenial),
    ControlJoin(SelectedCustodyDenial),
    DuplicateClaim,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryReleaseHeadReadDenial {
    ManifestEntryLimit {
        reference: ReleaseCustodyHeadBlockReferenceV1,
    },
    Media {
        reference: ReleaseCustodyHeadBlockReferenceV1,
        failure: RecoveryDiscoveryFailure,
    },
    MissingBytes {
        reference: ReleaseCustodyHeadBlockReferenceV1,
    },
    ResidentBoundExceeded {
        reference: ReleaseCustodyHeadBlockReferenceV1,
        required: u64,
        admitted: u64,
    },
    Allocation {
        reference: ReleaseCustodyHeadBlockReferenceV1,
        requested: u64,
        cause: std::collections::TryReserveError,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryReleaseHeadWalkDenial {
    Read(PhysicalRecoveryReleaseHeadReadDenial),
    Format(ReleaseCustodyHeadDenial),
    Root,
    DuplicateNode,
    BoundExceeded,
    /// The tree names more blocks than one holding the verified roster's
    /// heads can have. `admitted` is that ceiling; recovery sets no such limit.
    RosterBlockCeiling {
        observed: u64,
        admitted: u64,
    },
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    ResidentBoundExceeded {
        required: u64,
        admitted: u64,
    },
    EntryCountExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryReleaseHeadControlDenial {
    RouteOrder,
    HeadOrder,
    RequestCountOverflow,
    ManifestEntryLimit,
    ResidentBoundExceeded {
        required: u64,
        admitted: u64,
    },
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    ConflictingRequest {
        descriptor: PersistedRecordIdentity,
    },
    RouteMissing {
        record: PersistedRecordIdentity,
    },
    RouteMismatch {
        record: PersistedRecordIdentity,
    },
    DescriptorDecode {
        record: PersistedRecordIdentity,
        denial: BlobRecordDenial,
    },
    DescriptorKind {
        record: PersistedRecordIdentity,
    },
    DescriptorManifestMismatch {
        record: PersistedRecordIdentity,
    },
    ControlRead {
        record: PersistedRecordIdentity,
        denial: PhysicalRecoverySelectedRecordReadDenial,
    },
    FrameDigestMismatch {
        record: PersistedRecordIdentity,
    },
    WitnessMismatch {
        record: PersistedRecordIdentity,
        denial: SelectedCustodyDenial,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoverySelectedRecordReadDenial {
    InvalidRoute,
    ManifestEntryLimit,
    ResidentBoundExceeded,
    ManifestRead(RecoveryDiscoveryFailure),
    ManifestIntegrity(RecoveryIntegrityIngressRejection),
    ChunkRead {
        ordinal: u32,
        failure: RecoveryDiscoveryFailure,
    },
    ChunkIntegrity {
        ordinal: u32,
        denial: RecoveryIntegrityIngressRejection,
    },
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    InvalidPayload,
}
