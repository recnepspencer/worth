//! Original preparation failures survive the Native/Query handoff.
use crate::derived_relation::DerivedRelationReadDenial;
use crate::mvcc::{CompanionPreflightStop, PublicationCompanionRegistrationStop};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagedIndexSourcePreparationDenial {
    Source(DerivedRelationReadDenial),
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
