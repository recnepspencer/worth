use worth_store::physical_runtime::RecoveryDiscoveryFailure;
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

use crate::entry::PhysicalRecoveryPageAdmissionDenial;

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
    InvalidPage(PhysicalRedoTargetIdentity),
    ManifestEntryLimit,
    ByteLimit,
}

impl PageObservationFailure {
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
            Self::InvalidPage(target) => PhysicalRecoveryPageAdmissionDenial::InvalidPage(target),
            Self::ManifestEntryLimit => PhysicalRecoveryPageAdmissionDenial::ManifestEntryLimit,
            Self::ByteLimit => PhysicalRecoveryPageAdmissionDenial::ObservationByteLimit,
        }
    }
}
