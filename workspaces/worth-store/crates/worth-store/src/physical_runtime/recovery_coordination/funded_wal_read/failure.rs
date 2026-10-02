//! Borrowed diagnostics keep the actual filename and native charge inseparable.

use std::{ffi::OsStr, sync::Arc};

use worth_store_physical_backend::{
    ArtifactTreeFailure, RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryArtifact,
    RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure,
};
use worth_store_physical_format::RecordArtifactFile;

use super::super::PhysicalRecoveryObservationAllocationDenial as Denial;
use super::diagnostic_backing::WalReadDiagnosticStorage;

pub(super) type RawWalReadFailure = RecoveryDiscoveryAllocationFailure<Denial>;

/// Cloning shares one diagnostic allocation and its original pool reservation.
/// No owning backend error or filename can be extracted through this surface.
#[derive(Debug, Clone)]
pub struct FundedRecoveryWalReadFailure {
    pub(super) storage: FailureStorage,
}

#[derive(Debug, Clone)]
pub(super) enum FailureStorage {
    Inline { requested: usize, cause: Denial },
    SegmentLimit { observed: u64, admitted: u64 },
    Shared(Arc<WalReadDiagnosticStorage>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryWalArtifactView<'a> {
    Record(RecordArtifactFile),
    CurrentCheckpoint,
    WalDirectory,
    WalArtifact(&'a OsStr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryWalDiscoveryFailureView<'a> {
    EntryLimitExceeded {
        observed: u64,
        admitted: u64,
    },
    ByteLimitExceeded {
        observed: u64,
        admitted: u64,
        scope: RecoveryDiscoveryByteLimitScope,
    },
    Media {
        artifact: RecoveryWalArtifactView<'a>,
        failure: &'a ArtifactTreeFailure,
    },
    InvalidAddress {
        artifact: RecoveryWalArtifactView<'a>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryWalReadFailureView<'a> {
    Discovery(RecoveryWalDiscoveryFailureView<'a>),
    Allocation {
        artifact: RecoveryWalArtifactView<'a>,
        offset: u64,
        requested: usize,
        cause: &'a Denial,
    },
    BufferLengthMismatch {
        artifact: RecoveryWalArtifactView<'a>,
        offset: u64,
        requested: usize,
        observed: usize,
    },
}

impl FundedRecoveryWalReadFailure {
    pub fn diagnostic(&self) -> RecoveryWalReadFailureView<'_> {
        match &self.storage {
            FailureStorage::Inline { requested, cause } => RecoveryWalReadFailureView::Allocation {
                artifact: RecoveryWalArtifactView::WalDirectory,
                offset: 0,
                requested: *requested,
                cause,
            },
            FailureStorage::SegmentLimit { observed, admitted } => {
                RecoveryWalReadFailureView::Discovery(
                    RecoveryWalDiscoveryFailureView::EntryLimitExceeded {
                        observed: *observed,
                        admitted: *admitted,
                    },
                )
            }
            FailureStorage::Shared(storage) => {
                view(storage.failure.as_ref().expect("sealed failure"))
            }
        }
    }

    pub fn charged_bytes(&self) -> u64 {
        match &self.storage {
            FailureStorage::Inline { .. } | FailureStorage::SegmentLimit { .. } => 0,
            FailureStorage::Shared(storage) => storage.backing.bytes(),
        }
    }

    pub(super) fn inline(requested: usize, cause: Denial) -> Self {
        Self {
            storage: FailureStorage::Inline { requested, cause },
        }
    }

    pub(super) fn segment_limit(observed: u64, admitted: u64) -> Self {
        Self {
            storage: FailureStorage::SegmentLimit { observed, admitted },
        }
    }
}

impl PartialEq for FundedRecoveryWalReadFailure {
    fn eq(&self, other: &Self) -> bool {
        self.diagnostic() == other.diagnostic()
    }
}

impl Eq for FundedRecoveryWalReadFailure {}

fn view(failure: &RawWalReadFailure) -> RecoveryWalReadFailureView<'_> {
    match failure {
        RawWalReadFailure::Discovery(failure) => {
            RecoveryWalReadFailureView::Discovery(match failure {
                RecoveryDiscoveryFailure::EntryLimitExceeded { observed, admitted } => {
                    RecoveryWalDiscoveryFailureView::EntryLimitExceeded {
                        observed: *observed,
                        admitted: *admitted,
                    }
                }
                RecoveryDiscoveryFailure::ByteLimitExceeded {
                    observed,
                    admitted,
                    scope,
                } => RecoveryWalDiscoveryFailureView::ByteLimitExceeded {
                    observed: *observed,
                    admitted: *admitted,
                    scope: *scope,
                },
                RecoveryDiscoveryFailure::Media { artifact, failure } => {
                    RecoveryWalDiscoveryFailureView::Media {
                        artifact: artifact_view(artifact),
                        failure,
                    }
                }
                RecoveryDiscoveryFailure::InvalidAddress { artifact } => {
                    RecoveryWalDiscoveryFailureView::InvalidAddress {
                        artifact: artifact_view(artifact),
                    }
                }
            })
        }
        RawWalReadFailure::Allocation {
            artifact,
            offset,
            requested,
            cause,
        } => RecoveryWalReadFailureView::Allocation {
            artifact: artifact_view(artifact),
            offset: *offset,
            requested: *requested,
            cause,
        },
        RawWalReadFailure::BufferLengthMismatch {
            artifact,
            offset,
            requested,
            observed,
        } => RecoveryWalReadFailureView::BufferLengthMismatch {
            artifact: artifact_view(artifact),
            offset: *offset,
            requested: *requested,
            observed: *observed,
        },
    }
}

fn artifact_view(artifact: &RecoveryDiscoveryArtifact) -> RecoveryWalArtifactView<'_> {
    match artifact {
        RecoveryDiscoveryArtifact::Record(record) => RecoveryWalArtifactView::Record(*record),
        RecoveryDiscoveryArtifact::CurrentCheckpoint => RecoveryWalArtifactView::CurrentCheckpoint,
        RecoveryDiscoveryArtifact::WalDirectory => RecoveryWalArtifactView::WalDirectory,
        RecoveryDiscoveryArtifact::WalArtifact(name) => RecoveryWalArtifactView::WalArtifact(name),
    }
}

pub(super) fn filename_capacity(failure: &RawWalReadFailure) -> usize {
    let artifact = match failure {
        RawWalReadFailure::Discovery(
            RecoveryDiscoveryFailure::Media { artifact, .. }
            | RecoveryDiscoveryFailure::InvalidAddress { artifact },
        )
        | RawWalReadFailure::Allocation { artifact, .. }
        | RawWalReadFailure::BufferLengthMismatch { artifact, .. } => Some(artifact),
        RawWalReadFailure::Discovery(_) => None,
    };
    match artifact {
        Some(RecoveryDiscoveryArtifact::WalArtifact(name)) => name.capacity(),
        _ => 0,
    }
}
