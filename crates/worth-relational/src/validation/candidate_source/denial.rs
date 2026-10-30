//! Original preparation failures survive the Native/Query handoff.
use crate::derived_relation::DerivedRelationReadDenial;
use crate::mvcc::{CompanionPreflightStop, PublicationCompanionRegistrationStop};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagedIndexSourcePreparationDenial {
    Source(DerivedRelationReadDenial),
    Provider(super::ManagedIndexSourceProviderFailure),
    Publication(CompanionPreflightStop),
    Registration(PublicationCompanionRegistrationStop),
    MissingParticipant,
    IncompletePreparation,
    ForeignSource,
    RootUnavailable,
}

impl From<DerivedRelationReadDenial> for ManagedIndexSourcePreparationDenial {
    fn from(cause: DerivedRelationReadDenial) -> Self {
        Self::Source(cause)
    }
}

impl ManagedIndexSourcePreparationDenial {
    /// This exact Source posture had no available root; never means empty rows.
    pub fn source_unavailable(&self) -> bool {
        matches!(self, Self::RootUnavailable)
    }

    /// Borrow the provider's original owned failure without changing its custody.
    pub fn provider_failure(&self) -> Option<&super::ManagedIndexSourceProviderFailure> {
        match self {
            Self::Provider(cause) => Some(cause),
            _ => None,
        }
    }

    pub(super) fn execution_cause(
        &self,
    ) -> Option<crate::execution::RelationalExecutionDenialCause> {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        use crate::validation::data::{CustomInvariantInterruption, StructuralWalkDenialKind};

        match self {
            Self::Source(DerivedRelationReadDenial::Request(cause)) => Some(*cause),
            Self::Source(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::Interrupted(interruption),
            )) => Some(match interruption {
                CustomInvariantInterruption::Cancelled => Cause::Cancelled,
                CustomInvariantInterruption::DeadlineElapsed => Cause::DeadlineElapsed,
            }),
            Self::Source(DerivedRelationReadDenial::Structural(
                StructuralWalkDenialKind::MemoryAdmission(cause),
            )) => Some(crate::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(*cause),
            )),
            _ => None,
        }
    }
}
