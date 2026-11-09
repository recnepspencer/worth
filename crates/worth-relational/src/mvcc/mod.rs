pub(crate) use publication::BoundAuthoredCoupling;
pub(crate) use publication::CoupledSource;
mod observation;
pub(crate) mod publication;
pub use publication::RelationalPublicationReadDependencyDenial;
pub use publication::{
    PreparedMemoryRelationalCutover, RecordedRequiredOutputRole,
    RelationalPublicationAdoptionRequest, RelationalPublicationAppendix,
    RelationalPublicationBodySelection, ScopedRelationalPublication,
};
pub(crate) mod owned_graph_selection;
pub(crate) mod retained_entity_selection;
pub use owned_graph_selection::{
    RelationalOwnedGraphSelection, RelationalOwnedGraphSelectionPreparationError,
};
mod transaction;
#[cfg(test)]
pub(crate) use transaction::split_contacts;
pub(crate) mod validation;

pub use crate::runtime::{
    RelationalCancellationSource, RelationalCancellationToken, RelationalInterruptionBoundary,
    RelationalInterruptionCostCounters, RelationalInterruptionEvent, RelationalOperationControl,
    RelationalOperationInterruption,
};
pub use observation::RelationalBranchObservation;
pub(crate) use publication::PreparedCanonicalBranchMovement;
pub use publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use publication::{origin_denial, RequiredOutputPublicationAdmission};
pub use publication::{
    AuthoredOwnedMemberKind, CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage,
    CompanionCellEditStop, CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    DiscardedRelationalCommitCandidate, ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement,
    ManagedIndexSourceForkPreparation, ManagedIndexSourceObservation, ManagedIndexSourceRebuild,
    PendingCompanionRegistration, PerformedRelationalCommit, PreparedCompanionBranchCell,
    PreparedCompanionDerivedRoot, PreparedPublicationCompanionEffect,
    PreparedRelationalCommitCandidate, PublicationCompanionPreflight,
    PublicationCompanionRegistration, PublicationCompanionRegistrationPort,
    PublicationCompanionRegistrationStop, PublishRelationalCommit, RelationalCurrentOutputAdoption,
    RelationalPublicationCompanion, RelationalPublicationDeferred, RelationalPublicationDenial,
    RelationalPublicationDurabilityPosture, RelationalPublicationFailure,
    RelationalPublicationFailureKind, RelationalPublicationOutcome, RelationalPublicationPort,
    RelationalPublicationProjectionPosture, ReservedCompanionBranchCell,
    StaleRelationalBranchObservation,
};
pub(crate) use publication::{
    ManagedSourceHistoryOwner, ManagedSourceRootImage, PreparedIndexRefreshBasis,
    PreparedManagedIndexSourceFork, PreparedRelationalCandidateAdmissionStop,
    PreparedRelationalPublication, PreparedRelationalPublicationAccelerators,
    PublicationPreparationDenial,
};
pub use publication::{
    RelationalAuthoredCoupledCandidate, RelationalAuthoredCoupledOwners,
    RelationalAuthoredCoupledProducerAssembly, RelationalAuthoredCoupledTransaction,
    RelationalAuthoredRestoration, RelationalAuthoredRetirement, RelationalCoupledAssemblyFailure,
    RelationalCoupledAuthoredPublication, RelationalCoupledAuthoredSourceEffects,
    RelationalCoupledContribution, RelationalCoupledProducerPosture,
    RelationalDeclaredCoupledOwner, RelationalHistoricalOutputSelection,
    RelationalPerformedOutputPublications, RelationalPredecessorBasisDenial,
    RelationalPredecessorBasisOrigin, RelationalPredecessorInput,
    RelationalPreparedOutputPublications, RelationalRequiredOutputPublication,
    RelationalRequiredOutputRouteAbsence, RelationalRestoredProducerPrior,
    RequiredOutputProducingImplementation, RequiredOutputPublication,
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
pub use retained_entity_selection::{
    RelationalRetainedEntitySelection, RelationalRetainedEntitySelectionDenial,
    RelationalRetainedEntitySelectionPreparationError,
};
pub(crate) use transaction::commit_plan::bulk_reservations_for_plan;
pub(crate) use transaction::RelationalMaterializationTransactionMode;
pub(crate) use transaction::RelationalTransactionSavepoint;
pub use transaction::{
    BranchBoundRelationalTransaction, PreparedRelationalAspectScope,
    PreparedRelationalChangeSummary, PreparedRelationalChangeSummaryBudget,
    PreparedRelationalChangeSummaryDenial, PreparedRelationalRecordChange,
    PreparedRelationalRelationEndpoints, RelationalBranchTransactionAdmissionDenial,
    RelationalPreparationPort, RelationalTransactionEntityRead, RelationalTransactionFootprint,
    RelationalTransactionIntent, RelationalTransactionReadDenial, RelationalTransactionReadLocus,
    RelationalTransactionRelationRead, RelationalTransactionRelationValue,
    RelationalTransactionStagedMutations, RelationalTransactionStagingDenial,
    RelationalTransactionWriteLocus,
};
pub(crate) use validation::RelationalTransactionValidationInput;
pub use validation::{
    CustomInvariantExecutionReceipt, CustomInvariantNonApplicability,
    RelationalMutationInvariantEvidence, RelationalMutationProposalIdentity,
    ValidatedEntityStructTouch, ValidatedMutationFootprint, ValidatedMutationFootprintNotRequested,
    ValidatedMutationFootprintProjection, ValidatedMutationFootprintWork, ValidatedMutationTouch,
    ValidatedMutationTouchProjectionError, ValidatedMutationTouchProjectionWork,
    ValidatedMutationTouches, ValidatedRelationMutation, ValidatedRelationMutationFootprint,
    ValidatedRelationalProposal,
};

#[cfg(test)]
pub(crate) use publication::{native_birth_fixture, native_meaning_fixture};

#[cfg(test)]
pub(crate) use publication::native_roles_fixture;

pub(crate) use publication::require_graph_coverage;
