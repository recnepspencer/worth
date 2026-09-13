use worth_store_physical_integrity::PhysicalIntegrityRejection;

use crate::entry::PhysicalRecoveryRootProtocolDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryIntegrityIngressRejection {
    Absent,
    ConflictingDuplication { observed_sources: u64 },
    Integrity(PhysicalIntegrityRejection),
    NonCanonicalEncoding,
    MissingBoundedArtifact,
    SourceRangeOutsideObservation,
    ScopeMismatch,
    SourceIncarnationMismatch,
}

impl RecoveryIntegrityIngressRejection {
    pub(crate) const fn diagnostic(self) -> PhysicalRecoveryRootProtocolDenial {
        match self {
            Self::Absent | Self::MissingBoundedArtifact => {
                PhysicalRecoveryRootProtocolDenial::Absent
            }
            Self::ConflictingDuplication { observed_sources } => {
                PhysicalRecoveryRootProtocolDenial::ConflictingDuplication { observed_sources }
            }
            Self::Integrity(rejection) => PhysicalRecoveryRootProtocolDenial::Integrity(rejection),
            Self::NonCanonicalEncoding => PhysicalRecoveryRootProtocolDenial::NonCanonicalEncoding,
            Self::ScopeMismatch | Self::SourceRangeOutsideObservation => {
                PhysicalRecoveryRootProtocolDenial::ScopeMismatch
            }
            Self::SourceIncarnationMismatch => {
                PhysicalRecoveryRootProtocolDenial::SourceIncarnationMismatch
            }
        }
    }
}
