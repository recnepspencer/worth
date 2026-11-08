pub(crate) use application_publication::BoundAuthoredCoupling;
pub(crate) use application_publication::CoupledSource;
mod application_publication;
mod authority;
mod preparation_denial;
pub use application_publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use application_publication::{origin_denial, RequiredOutputPublicationAdmission};
pub use application_publication::{
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledOwners,
    RelationalAuthoredCoupledProducerAssembly, RelationalAuthoredCoupledTransaction,
    RelationalAuthoredRestoration, RelationalAuthoredRetirement, RelationalCoupledAssemblyFailure,
    RelationalCoupledAuthoredPublication, RelationalDeclaredCoupledOwner,
    RelationalHistoricalOutputSelection, RelationalPerformedOutputPublications,
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin, RelationalPredecessorInput,
    RelationalPreparedOutputPublications, RelationalRequiredOutputPublication,
    RelationalRequiredOutputRouteAbsence, RelationalRestoredProducerPrior,
    RequiredOutputProducingImplementation, RequiredOutputPublication,
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
pub(crate) use preparation_denial::PublicationPreparationDenial;
mod candidate;
mod candidate_consumption;
mod candidate_preparation;
mod companion;
mod current_read;
pub use current_read::RelationalPublicationReadDependencyDenial;
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
    ManagedSourceHistoryOwner, ManagedSourceRootImage, PreparedManagedIndexSourceFork,
};
pub use companion::{
    CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage, CompanionCellEditStop,
    CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement, ManagedIndexSourceForkPreparation,
    ManagedIndexSourceObservation, ManagedIndexSourceRebuild, PendingCompanionRegistration,
    PreparedCompanionBranchCell, PreparedCompanionDerivedRoot, PreparedPublicationCompanionEffect,
    PublicationCompanionPreflight, PublicationCompanionRegistration,
    PublicationCompanionRegistrationPort, PublicationCompanionRegistrationStop,
    RelationalPublicationCompanion, ReservedCompanionBranchCell,
};
pub use outcome::{
    PerformedRelationalCommit, PublishRelationalCommit, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationProjectionPosture, StaleRelationalBranchObservation,
};
pub(crate) use port::PreparedCanonicalBranchMovement;
pub use port::RelationalPublicationPort;
pub use port::{PreparedMemoryRelationalCutover, ScopedRelationalPublication};

#[cfg(test)]
pub(crate) use application_publication::{native_birth_fixture, native_meaning_fixture};

#[cfg(test)]
pub(crate) use application_publication::native_roles_fixture;

pub(crate) use application_publication::require_graph_coverage;
