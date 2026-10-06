use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

use crate::entry::{
    HistoricalDropAdmissionStage, PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryPageAdmissionDenial,
};
use crate::orchestration::reader_limit::ReaderBytes;
use crate::orchestration::recovery_budget::ExceededRecoveryLimit;

/// A limit page observation ran out of. It says nothing about the media.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PageLimit {
    /// Refused by recovery's own allowance, in recovery's counts.
    Recovery(ExceededRecoveryLimit),
    /// The observation bytes the reader was handed ran out, in its counts.
    Reader(ReaderBytes),
}

impl PageLimit {
    /// The limit in recovery's counts, where the reader was handed what was
    /// left of recovery's declared observation bytes. `None` where no limit
    /// can state the reader's counts in recovery's.
    pub(crate) fn in_recovery(
        self,
        limits: &PhysicalRecoveryLimitDeclaration,
    ) -> Option<PhysicalRecoveryLimitFailure> {
        match self {
            Self::Recovery(limit) => Some(limit.into()),
            Self::Reader(bytes) => bytes.in_recovery(limits).map(Into::into),
        }
    }
}

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
    Limit(PageLimit),
    /// A count observation keeps went past every count. No limit admits it.
    CountOverflow,
}

impl PageObservationFailure {
    /// A failed media read: a limit where the reader's observation bytes ran
    /// out, damage everywhere else.
    pub(crate) fn media(
        target: Option<PhysicalRedoTargetIdentity>,
        failure: RecoveryDiscoveryFailure,
    ) -> Self {
        match ReaderBytes::of(&failure) {
            Some(bytes) => Self::Limit(PageLimit::Reader(bytes)),
            None => Self::Media { target, failure },
        }
    }

    /// The damage, as evidence; a limit is no evidence about the media.
    pub(crate) fn evidence(self) -> Result<PhysicalRecoveryPageAdmissionDenial, PageLimit> {
        Ok(match self {
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
            Self::CountOverflow => PhysicalRecoveryPageAdmissionDenial::CountOverflow,
            Self::Limit(limit) => return Err(limit),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestration::reader_limit::refused_past;
    use worth_store::physical_runtime::{FilesystemObservationBound, RecoveryDiscoveryArtifact};

    #[test]
    fn only_the_readers_exhausted_observation_bytes_are_a_limit() {
        let target = Some(PhysicalRedoTargetIdentity::InlinePage {
            segment: 1,
            page: 2,
            generation: 3,
        });
        let past = refused_past(FilesystemObservationBound::ObservationBytes, 65_537, 65_536);
        let PageObservationFailure::Limit(limit) =
            PageObservationFailure::media(target, past.clone())
        else {
            panic!("exhausted observation bytes are a limit");
        };
        // Handed 65,536 of recovery's 65,540, the reader needed 65,537.
        let limit = limit
            .in_recovery(&PhysicalRecoveryLimitDeclaration::observing_for_test(
                65_540,
            ))
            .unwrap();
        assert_eq!(
            (limit.dimension(), limit.observed(), limit.admitted()),
            (
                crate::entry::PhysicalRecoveryLimitDimension::ObservationBytes,
                65_541,
                65_540
            ),
        );
        assert_eq!(
            PageObservationFailure::Limit(PageLimit::Reader(ReaderBytes::of(&past).unwrap()))
                .evidence(),
            Err(PageLimit::Reader(ReaderBytes::of(&past).unwrap())),
        );
        for failure in [
            // The reader counts no reads: a refused read is past every count.
            refused_past(FilesystemObservationBound::Reads, 1, 0),
            // The artifact outgrew the ceiling of its own read.
            refused_past(FilesystemObservationBound::RequestedBytes, 65_537, 65_536),
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
