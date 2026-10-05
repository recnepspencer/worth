use worth_store::physical_runtime::{RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

use crate::entry::{HistoricalDropAdmissionStage, PhysicalRecoveryPageAdmissionDenial};

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
}

impl PageObservationFailure {
    /// A failed media read. Only an exhausted observation budget is a limit:
    /// the bytes observation may read, or the addressed reads its remaining
    /// manifest entries admit. A read that exceeded the ceiling requested for
    /// its one artifact found an artifact larger than its format admits, which
    /// is damage.
    pub(crate) fn media(
        target: Option<PhysicalRedoTargetIdentity>,
        failure: RecoveryDiscoveryFailure,
    ) -> Self {
        match failure {
            RecoveryDiscoveryFailure::ByteLimitExceeded {
                scope: RecoveryDiscoveryByteLimitScope::Observation,
                ..
            } => Self::ByteLimit,
            RecoveryDiscoveryFailure::EntryLimitExceeded { .. } => Self::ManifestEntryLimit,
            failure => Self::Media { target, failure },
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store::physical_runtime::RecoveryDiscoveryArtifact;

    #[test]
    fn only_an_exhausted_observation_budget_is_a_limit() {
        let oversized = |scope| RecoveryDiscoveryFailure::ByteLimitExceeded {
            observed: 65_537,
            admitted: 65_536,
            scope,
        };
        let target = Some(PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 2,
            generation: 3,
        });
        assert_eq!(
            PageObservationFailure::media(
                target,
                oversized(RecoveryDiscoveryByteLimitScope::Observation)
            ),
            PageObservationFailure::ByteLimit,
        );
        // The reader raises this only once its addressed reads ran out.
        assert_eq!(
            PageObservationFailure::media(
                target,
                RecoveryDiscoveryFailure::EntryLimitExceeded {
                    observed: 1,
                    admitted: 0,
                }
            ),
            PageObservationFailure::ManifestEntryLimit,
        );
        for failure in [
            // The artifact outgrew the ceiling of its own read.
            oversized(RecoveryDiscoveryByteLimitScope::Requested),
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
