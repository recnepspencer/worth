use std::path::{Path, PathBuf};

use super::RecoveryObserverCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryObserverWalTopologyDenial {
    MalformedFrame,
    NonContiguousSegment,
    DuplicateSegment,
    GenerationMismatch,
    SegmentIdentityMismatch,
    NonContiguousLsn,
    LsnGap,
    LsnOverlap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryObserverObservationDenial {
    DirectoryEntryLimit { observed: u64, admitted: u64 },
    DirectoryLimit { observed: u64, admitted: u64 },
    ArtifactLimit { observed: u64, admitted: u64 },
    ByteLimit { observed: u64, admitted: u64 },
    SymbolicLink,
    UnsupportedFileType,
    NonUnicodePath,
    ArtifactChanged,
    EvidenceBufferAllocationFailed,
    Media(std::io::ErrorKind),
    WalTopology(RecoveryObserverWalTopologyDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryObserverObservationFailure {
    denial: RecoveryObserverObservationDenial,
    counters: RecoveryObserverCounters,
    path: Option<PathBuf>,
}

impl RecoveryObserverObservationFailure {
    pub(super) fn at_path(
        denial: RecoveryObserverObservationDenial,
        counters: RecoveryObserverCounters,
        path: &Path,
    ) -> Self {
        Self {
            denial,
            counters,
            path: Some(path.to_path_buf()),
        }
    }

    pub const fn denial(&self) -> RecoveryObserverObservationDenial {
        self.denial
    }

    pub const fn counters(&self) -> RecoveryObserverCounters {
        self.counters
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}
