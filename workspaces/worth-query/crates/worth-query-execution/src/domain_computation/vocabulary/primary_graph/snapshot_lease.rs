//! Moved whole from `domain_computation::primary_graph::application_attempt::snapshot_lease`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisReleaseOutcome;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationSnapshotLeaseDenial {
    ForeignRuntime,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationSnapshotRelease {
    NativeSnapshotReleased,
    SharedBasisRelease(WorthQueryApplicationBasisReleaseOutcome),
}

impl WorthQueryApplicationSnapshotRelease {
    pub(in crate::domain_computation) const fn snapshot_released(self) -> bool {
        match self {
            Self::NativeSnapshotReleased => true,
            Self::SharedBasisRelease(outcome) => outcome.snapshot_released(),
        }
    }

    pub(in crate::domain_computation) const fn custody_released(self) -> bool {
        match self {
            Self::NativeSnapshotReleased => true,
            Self::SharedBasisRelease(outcome) => outcome.custody_released(),
        }
    }
}
