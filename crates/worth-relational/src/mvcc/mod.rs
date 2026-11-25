pub use crate::history::data::{RelationalCallerAssociation, RelationalCallerAssociationRefusal};
pub(crate) use publication::BoundAuthoredCoupling;
pub(crate) use publication::CoupledSource;
pub(crate) use worth_relational_branch::internal_port::mvcc::observation;
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
#[cfg(test)]
pub(crate) use transaction::split_contacts;
pub(crate) mod validation;

pub(crate) use crate::durability::RecordedReplayResultSchema;
pub use crate::runtime::{
    RelationalCancellationSource, RelationalCancellationToken, RelationalInterruptionBoundary,
    RelationalInterruptionCostCounters, RelationalInterruptionEvent, RelationalOperationControl,
    RelationalOperationInterruption,
};
pub use observation::RelationalBranchObservation;
pub use publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use publication::RequiredOutputPublicationAdmission;
pub use publication::{
    AuthoredOwnedMemberKind, CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage,
    CompanionCellEditStop, CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    DiscardedRelationalCommitCandidate, ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement,
    ManagedIndexSourceCellPlacement, ManagedIndexSourceObservation, ManagedIndexSourceRebuild,
    PerformedRelationalCommit, PreparedCompanionBranchCell, PreparedCompanionDerivedRoot,
    PreparedPublicationCompanionEffect, PreparedRelationalCommitCandidate,
    PublicationCompanionPreflight, PublicationCompanionRegistrationStop, PublishRelationalCommit,
    RelationalCurrentOutputAdoption, RelationalPublicationCompanion, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationProjectionPosture, ReservedCompanionBranchCell,
    StaleRelationalBranchObservation,
};
pub(crate) use publication::{PreparedManagedIndexSourceFork, PublicationPreparationDenial};
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

pub(crate) use validation::ValidatedProposalState;

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

#[cfg(test)]
pub(crate) use transaction::normalized_plan as plan_copy_admission;

#[cfg(test)]
pub(crate) use transaction::normalized_plan::OwnedNormalizedPlan;

#[cfg(test)]
pub(crate) use transaction::OwnedMergePlan;
