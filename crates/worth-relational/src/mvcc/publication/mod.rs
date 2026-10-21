mod application_publication;
mod authority;
pub use application_publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use application_publication::{origin_denial, RequiredOutputPublicationAdmission};
pub use application_publication::{
    RelationalRequiredOutputPublication, RelationalRequiredOutputRouteAbsence,
    RequiredOutputProducingImplementation, RequiredOutputPublication,
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
mod candidate;
mod candidate_consumption;
mod candidate_preparation;
mod companion;
mod outcome;
mod port;
mod validation;

pub(crate) use authority::{
    PreparedIndexRefreshBasis, PreparedRelationalPublication,
    PreparedRelationalPublicationAccelerators,
};
pub(crate) use candidate::{CandidatePayload, PreparedRelationalCandidateAdmissionStop};
pub use candidate::{DiscardedRelationalCommitCandidate, PreparedRelationalCommitCandidate};
pub(crate) use companion::{
    CandidateCompanionBinding, CompanionRegistrationEpoch, CompanionRegistry,
};
pub use companion::{
    CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage, CompanionCellEditStop,
    CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    PendingCompanionRegistration, PreparedCompanionBranchCell, PreparedCompanionDerivedRoot,
    PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    PublicationCompanionRegistration, PublicationCompanionRegistrationPort,
    PublicationCompanionRegistrationStop, RelationalPublicationCompanion,
    ReservedCompanionBranchCell,
};
pub use outcome::{
    PerformedRelationalCommit, PublishRelationalCommit, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationProjectionPosture, StaleRelationalBranchObservation,
};
pub(crate) use port::PreparedCanonicalBranchMovement;
pub use port::RelationalPublicationPort;

#[cfg(test)]
pub(crate) use application_publication::{native_birth_fixture, native_meaning_fixture};

#[cfg(test)]
pub(crate) use application_publication::native_roles_fixture;
