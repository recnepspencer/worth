pub(crate) use application_publication::BoundAuthoredCoupling;
pub(crate) use application_publication::CoupledSource;
pub(crate) mod application_publication;
pub(crate) mod authority;
mod caller_association;
mod preparation_denial;
pub use application_publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use application_publication::{
    origin_denial, RequiredOutputInputs, RequiredOutputPublicationAdmission,
};
pub(crate) fn custody_stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError {
    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::execution(
        cause.into().into(),
    )
}
pub use application_publication::{
    AuthoredOwnedMemberKind, ManagedIndexParentReadFootprint, RecordedRequiredOutputRole,
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledOwners,
    RelationalAuthoredCoupledProducerAssembly, RelationalAuthoredCoupledTransaction,
    RelationalAuthoredRestoration, RelationalAuthoredRetirement, RelationalCoupledAppendTouches,
    RelationalCoupledAssemblyFailure, RelationalCoupledAuthoredPublication,
    RelationalCoupledAuthoredSourceEffects, RelationalCoupledChainOverlay,
    RelationalCoupledContribution, RelationalCoupledProducerContinuation,
    RelationalCoupledProducerPosture, RelationalDeclaredCoupledOwner,
    RelationalHistoricalOutputSelection, RelationalPerformedOutputPublications,
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin, RelationalPredecessorInput,
    RelationalPreparedOutputPublications, RelationalPublicationAdoptionRequest,
    RelationalPublicationAppendix, RelationalPublicationBodySelection,
    RelationalRequiredOutputInput, RelationalRequiredOutputPublication,
    RelationalRequiredOutputRouteAbsence, RelationalRestoredProducerPrior,
    RequiredOutputProducingImplementation, RequiredOutputPublication,
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
pub(crate) use preparation_denial::PublicationPreparationDenial;
pub(crate) mod candidate;
pub(crate) mod companion;
mod current_read;
pub(crate) mod read_dependency_denial;
pub use current_read::RelationalPublicationReadDependencyDenial;
mod movement;
pub(crate) mod outcome;
pub(crate) mod port;
pub(crate) mod validation;

pub(crate) use authority::{
    PreparedRelationalPublication, PreparedRelationalPublicationAccelerators,
};
pub(crate) use candidate::{CandidatePayload, PreparedRelationalCandidateAdmissionStop};
pub use candidate::{
    DiscardedRelationalCommitCandidate, PreparedRelationalCommitCandidate,
    RelationalCurrentOutputAdoption,
};
pub(crate) use companion::{
    CandidateCompanionBinding, CompanionRegistrationEpoch, CompanionRegistry,
    PreparedManagedIndexSourceFork,
};
pub use companion::{
    CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage, CompanionCellEditStop,
    CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement, ManagedIndexSourceForkPreparation,
    ManagedIndexSourceObservation, ManagedIndexSourceRebuild, PreparedCompanionBranchCell,
    PreparedCompanionDerivedRoot, PreparedPublicationCompanionEffect,
    PublicationCompanionPreflight, PublicationCompanionRegistrationStop,
    RelationalPublicationCompanion, ReservedCompanionBranchCell,
};
pub(crate) use movement::PreparedCanonicalBranchMovement;
pub use outcome::{
    PerformedRelationalCommit, PublishRelationalCommit, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationProjectionPosture, StaleRelationalBranchObservation,
};

#[cfg(test)]
pub(crate) use application_publication::native_birth_fixture;

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub(crate) use application_publication::require_graph_coverage;
