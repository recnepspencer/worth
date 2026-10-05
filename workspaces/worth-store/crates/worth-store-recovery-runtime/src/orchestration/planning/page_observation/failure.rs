use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

use crate::entry::{HistoricalDropAdmissionStage, PhysicalRecoveryPageAdmissionDenial};
pub(crate) use crate::orchestration::reader_limit::ReaderLimit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PageObservationFailure {
    Media {
        target: Option<PhysicalRedoTargetIdentity>,
        failure: RecoveryDiscoveryFailure,
    },
    MissingArtifact {
        target: Option<PhysicalRedoTargetIdentity>,
        artifact: RecordArtifactFile,
    },
    InvalidManifest {
        target: Option<PhysicalRedoTargetIdentity>,
        artifact: RecordArtifactFile,
    },
    Integrity {
        artifact: RecordArtifactFile,
        denial: crate::entry::PhysicalRecoveryRootProtocolDenial,
    },
    InvalidTarget(PhysicalRedoTargetIdentity),
    HistoricalDrop {
        operation: [u8; 32],
        stage: HistoricalDropAdmissionStage,
        target: Option<PhysicalRedoTargetIdentity>,
    },
    AbsentExtentBelowFrontier {
        target: PhysicalRedoTargetIdentity,
        next_extent: u64,
    },
    MaterializedExtentChunkCount {
        target: PhysicalRedoTargetIdentity,
        admitted_chunk_count: u32,
    },
    MaterializedExtentCoordinate(PhysicalRedoTargetIdentity),
    InvalidPage(PhysicalRedoTargetIdentity),
    ManifestEntryLimit,
    ByteLimit,
    /// The ordered walk outgrew the scratch admitted as staging bytes. It
    /// needed at least this many; a step that knows only that it needed more
    /// than it had left says zero.
    StagingByteLimit {
        at_least: u64,
    },
}

impl PageObservationFailure {
    /// A failed media read: a limit where `ReaderLimit` says so, damage
    /// everywhere else.
    pub(crate) fn media(
        target: Option<PhysicalRedoTargetIdentity>,
        failure: RecoveryDiscoveryFailure,
    ) -> Self {
        match ReaderLimit::of(&failure) {
            Some(ReaderLimit::ObservationBytes { .. }) => Self::ByteLimit,
            Some(ReaderLimit::Reads { .. }) => Self::ManifestEntryLimit,
            None => Self::Media { target, failure },
        }
    }

    pub(crate) fn evidence(self) -> PhysicalRecoveryPageAdmissionDenial {
        match self {
            Self::Media { target, failure } => {
                PhysicalRecoveryPageAdmissionDenial::Media { target, failure }
            }
            Self::MissingArtifact { target, artifact } => {
                PhysicalRecoveryPageAdmissionDenial::MissingArtifact { target, artifact }
            }
            Self::InvalidManifest { target, artifact } => {
                PhysicalRecoveryPageAdmissionDenial::InvalidManifest { target, artifact }
            }
            Self::Integrity { artifact, denial } => {
                PhysicalRecoveryPageAdmissionDenial::Integrity { artifact, denial }
            }
            Self::InvalidTarget(target) => {
                PhysicalRecoveryPageAdmissionDenial::InvalidTarget(target)
            }
            Self::HistoricalDrop {
                operation,
                stage,
                target,
            } => PhysicalRecoveryPageAdmissionDenial::HistoricalDrop {
                operation,
                stage,
                target,
            },
            Self::AbsentExtentBelowFrontier {
                target,
                next_extent,
            } => PhysicalRecoveryPageAdmissionDenial::AbsentExtentBelowFrontier {
                target,
                next_extent,
            },
            Self::MaterializedExtentChunkCount {
                target,
                admitted_chunk_count,
            } => PhysicalRecoveryPageAdmissionDenial::MaterializedExtentChunkCount {
                target,
                admitted_chunk_count,
            },
            Self::MaterializedExtentCoordinate(target) => {
                PhysicalRecoveryPageAdmissionDenial::MaterializedExtentCoordinate(target)
            }
            Self::InvalidPage(target) => PhysicalRecoveryPageAdmissionDenial::InvalidPage(target),
            Self::ManifestEntryLimit => PhysicalRecoveryPageAdmissionDenial::ManifestEntryLimit,
            Self::ByteLimit => PhysicalRecoveryPageAdmissionDenial::ObservationByteLimit,
            Self::StagingByteLimit { .. } => PhysicalRecoveryPageAdmissionDenial::StagingByteLimit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::reader_limit::refused_past;
    use worth_store::physical_runtime::{FilesystemObservationBound, RecoveryDiscoveryArtifact};

    #[test]
    fn only_an_exhausted_observation_budget_is_a_limit() {
        let oversized = |bound| refused_past(bound, 65_537, 65_536);
        let target = Some(PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 2,
            generation: 3,
        });
        assert_eq!(
            PageObservationFailure::media(
                target,
                oversized(FilesystemObservationBound::ObservationBytes)
            ),
            PageObservationFailure::ByteLimit,
        );
        // The reader raises this only once its addressed reads ran out.
        assert_eq!(
            PageObservationFailure::media(
                target,
                refused_past(FilesystemObservationBound::Reads, 1, 0)
            ),
            PageObservationFailure::ManifestEntryLimit,
        );
        for failure in [
            // The artifact outgrew the ceiling of its own read.
            oversized(FilesystemObservationBound::RequestedBytes),
            RecoveryDiscoveryFailure::InvalidAddress {
                artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
            },
        ] {
            assert_eq!(
                PageObservationFailure::media(target, failure.clone()),
                PageObservationFailure::Media { target, failure },
            );
        }
    }
}
