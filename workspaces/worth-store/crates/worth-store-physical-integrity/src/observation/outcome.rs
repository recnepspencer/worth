use crate::validation::{PhysicalArtifactScope, PhysicalIntegrityRejection};
use worth_foundational::PhysicalIntegrityPosture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityObservationOutcome {
    Intact(PhysicalArtifactScope),
    Rejected(PhysicalIntegrityRejection),
}

impl PhysicalIntegrityObservationOutcome {
    pub const fn scope(self) -> PhysicalArtifactScope {
        match self {
            Self::Intact(scope) => scope,
            Self::Rejected(rejection) => rejection.scope(),
        }
    }

    pub const fn foundational_posture(self) -> PhysicalIntegrityPosture {
        match self {
            Self::Intact(_) => PhysicalIntegrityPosture::Intact,
            Self::Rejected(PhysicalIntegrityRejection::Damaged(_)) => {
                PhysicalIntegrityPosture::Damaged
            }
            Self::Rejected(PhysicalIntegrityRejection::Unsupported(_)) => {
                PhysicalIntegrityPosture::Unsupported
            }
            Self::Rejected(PhysicalIntegrityRejection::Unknown(_)) => {
                PhysicalIntegrityPosture::Unknown
            }
            Self::Rejected(PhysicalIntegrityRejection::Indeterminate(_)) => {
                PhysicalIntegrityPosture::Indeterminate
            }
        }
    }
}
