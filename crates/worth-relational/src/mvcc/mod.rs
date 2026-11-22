pub(crate) use publication::BoundAuthoredCoupling;
pub(crate) use publication::CoupledSource;
pub(crate) mod observation;
pub use crate::history::data::{RelationalCallerAssociation, RelationalCallerAssociationRefusal};
pub(crate) mod publication;
pub use publication::port::{
    PreparedMemoryRelationalCutover, RelationalPublicationPort, ScopedRelationalPublication,
};
pub use publication::RelationalPublicationReadDependencyDenial;
pub use publication::{
    RecordedRequiredOutputRole, RelationalPublicationAdoptionRequest,
    RelationalPublicationAppendix, RelationalPublicationBodySelection,
};
pub(crate) mod owned_graph_selection;
pub(crate) mod retained_entity_selection;
pub use owned_graph_selection::{
    RelationalOwnedGraphSelection, RelationalOwnedGraphSelectionPreparationError,
};
pub(crate) mod transaction;
pub(crate) use transaction::normalized_plan as plan_copy_admission;
pub(crate) use transaction::normalized_plan::OwnedNormalizedPlan;
#[cfg(test)]
pub(crate) use transaction::split_contacts;
pub(crate) use transaction::OwnedMergePlan;
pub(crate) mod validation;

pub(crate) use crate::durability::RecordedReplayResultSchema;
pub use crate::runtime::{
    RelationalCancellationSource, RelationalCancellationToken, RelationalInterruptionBoundary,
    RelationalInterruptionCostCounters, RelationalInterruptionEvent, RelationalOperationControl,
    RelationalOperationInterruption,
};
pub use observation::RelationalBranchObservation;
pub(crate) use publication::PreparedCanonicalBranchMovement;
pub use publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use publication::{
    origin_denial, RequiredOutputInputs, RequiredOutputPublicationAdmission,
};
pub use publication::{
    AuthoredOwnedMemberKind, CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage,
    CompanionCellEditStop, CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    DiscardedRelationalCommitCandidate, ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement,
    ManagedIndexSourceForkPreparation, ManagedIndexSourceObservation, ManagedIndexSourceRebuild,
    PerformedRelationalCommit, PreparedCompanionBranchCell, PreparedCompanionDerivedRoot,
    PreparedPublicationCompanionEffect, PreparedRelationalCommitCandidate,
    PublicationCompanionPreflight, PublicationCompanionRegistrationStop, PublishRelationalCommit,
    RelationalCurrentOutputAdoption, RelationalPublicationCompanion, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationProjectionPosture, ReservedCompanionBranchCell,
    StaleRelationalBranchObservation,
};
pub(crate) use publication::{
    PreparedManagedIndexSourceFork, PreparedRelationalCandidateAdmissionStop,
    PreparedRelationalPublication, PreparedRelationalPublicationAccelerators,
    PublicationPreparationDenial,
};
pub use publication::{
    RelationalAuthoredCoupledOwners, RelationalAuthoredRestoration, RelationalAuthoredRetirement,
    RelationalCoupledAssemblyFailure, RelationalCoupledAuthoredPublication,
    RelationalCoupledAuthoredSourceEffects, RelationalCoupledContribution,
    RelationalCoupledProducerPosture, RelationalDeclaredCoupledOwner,
    RelationalHistoricalOutputSelection, RelationalPerformedOutputPublications,
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin, RelationalPredecessorInput,
    RelationalPreparedOutputPublications, RelationalRequiredOutputInput,
    RelationalRequiredOutputPublication, RelationalRequiredOutputRouteAbsence,
    RelationalRestoredProducerPrior, RequiredOutputProducingImplementation,
    RequiredOutputPublication, RequiredOutputPublicationMeaning, RequiredOutputRolePosture,
    RequiredOutputRoleTarget,
};
pub use retained_entity_selection::{
    RelationalRetainedEntitySelection, RelationalRetainedEntitySelectionDenial,
    RelationalRetainedEntitySelectionPreparationError,
};
pub(crate) use transaction::RelationalMaterializationTransactionMode;
pub(crate) use transaction::RelationalTransactionSavepoint;
pub(crate) use transaction::TransactionSchemaOrigin;
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
    CustomInvariantExecutionReceipt, CustomInvariantNonApplicability, RecordedMemberRestoration,
    RecordedMemberRestorationRefusal, RelationalMutationInvariantEvidence,
    RelationalMutationProposalIdentity, ValidatedEntityStructTouch, ValidatedMutationFootprint,
    ValidatedMutationFootprintNotRequested, ValidatedMutationFootprintProjection,
    ValidatedMutationFootprintWork, ValidatedMutationTouch, ValidatedMutationTouchProjectionError,
    ValidatedMutationTouchProjectionWork, ValidatedMutationTouches, ValidatedRelationMutation,
    ValidatedRelationMutationFootprint, ValidatedRelationalProposal,
};

#[cfg(test)]
pub(crate) use publication::native_birth_fixture;

pub(crate) use validation::ValidatedProposalState;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
pub(crate) use publication::require_graph_coverage;

pub use publication::companion::registration::port::{
    PendingCompanionRegistration, PublicationCompanionRegistration,
    PublicationCompanionRegistrationPort, PublicationCompanionRetirementDenial,
    RetiredPublicationCompanionRegistration,
};

pub use publication::application_publication::coupled_assembly::{
    ManagedIndexParentReadFootprint, RelationalAuthoredCoupledCandidate,
    RelationalAuthoredCoupledProducerAssembly, RelationalAuthoredCoupledTransaction,
    RelationalCoupledAppendTouches, RelationalCoupledChainOverlay,
    RelationalCoupledOverlayEndpoints, RelationalCoupledOverlayRelation,
    RelationalCoupledProducerContinuation,
};
