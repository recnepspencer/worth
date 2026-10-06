//! Causes from C8's ordered release admission, never release authority.

use worth_store_physical_format::ReleaseCheckpointCertificateDenial;
use worth_store_recovery_physics::{
    EffectiveReleaseHeadDenial, OrderedHistoricalReleaseCustodyDenial,
    PendingWalReleaseCustodyDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryOrderedReleaseJoin {
    HistoricalOperation,
    ReleasedEdge,
    Descriptor,
    SampledWalMember,
    AdmittedWalFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryOrderedReleaseStorage {
    BatchRoster,
    HeadReplayRoster,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryOrderedReleaseDenial {
    CustodyPosture,
    ResidentBasis,
    MissingCheckpoint,
    MissingReleases,
    MissingHistory,
    RosterBinding,
    /// The block's cause names staging's limit and its counts, where a
    /// count can state it.
    StagingBoundExceeded,
    /// The block's cause names recovery memory's limit and its counts.
    ResidentBoundExceeded,
    RosterAllocation {
        storage: PhysicalRecoveryOrderedReleaseStorage,
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    OperationJoin {
        operation: [u8; 32],
        join: PhysicalRecoveryOrderedReleaseJoin,
    },
    WalFate {
        operation: [u8; 32],
        cause: ReleaseCheckpointCertificateDenial,
    },
    Batch {
        operation: [u8; 32],
        edge_index: usize,
        cause: PendingWalReleaseCustodyDenial,
    },
    PendingAttachment(PendingWalReleaseCustodyDenial),
    CompletedCustody(OrderedHistoricalReleaseCustodyDenial),
    EffectiveHeads(EffectiveReleaseHeadDenial),
}
