mod observation;
pub use crate::history::data::{RelationalCallerAssociation, RelationalCallerAssociationRefusal};
pub(crate) mod publication;
mod transaction;
pub(crate) use transaction::normalized_plan as plan_copy_admission;
pub(crate) use transaction::normalized_plan::OwnedNormalizedPlan;
#[cfg(test)]
pub(crate) use transaction::split_contacts;
pub(crate) use transaction::OwnedMergePlan;
pub(crate) mod validation;

pub use crate::runtime::{
    RelationalCancellationSource, RelationalCancellationToken, RelationalInterruptionBoundary,
    RelationalInterruptionCostCounters, RelationalInterruptionEvent, RelationalOperationControl,
    RelationalOperationInterruption,
};
pub use observation::RelationalBranchObservation;
pub(crate) use observation::RelationalObservationRetirement;
pub(crate) use observation::RelationalRetainedObservationShares;
pub(crate) use publication::PreparedCanonicalBranchMovement;
pub use publication::RelationalInstalledApplicationPublicationAuthority;
pub(crate) use publication::{origin_denial, RequiredOutputPublicationAdmission};
pub use publication::{
    CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage, CompanionCellEditStop,
    CompanionDerivedRootCleanup, CompanionDerivedRootInstalled,
    CompanionDerivedRootPreparationStop, CompanionDerivedRootStopped, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    DiscardedRelationalCommitCandidate, ManagedIndexPublicCache, ManagedIndexPublicCacheRetirement,
    ManagedIndexSourceForkPreparation, ManagedIndexSourceObservation, ManagedIndexSourceRebuild,
    PendingCompanionRegistration, PerformedRelationalCommit, PreparedCompanionBranchCell,
    PreparedCompanionDerivedRoot, PreparedPublicationCompanionEffect,
    PreparedRelationalCommitCandidate, PublicationCompanionPreflight,
    PublicationCompanionRegistration, PublicationCompanionRegistrationPort,
    PublicationCompanionRegistrationStop, PublicationCompanionRetirementDenial,
    PublishRelationalCommit, RelationalPublicationCompanion, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationDurabilityPosture,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
    RelationalPublicationPort, RelationalPublicationProjectionPosture, ReservedCompanionBranchCell,
    RetiredPublicationCompanionRegistration, StaleRelationalBranchObservation,
};
pub(crate) use publication::{
    ManagedSourceHistoryOwner, ManagedSourceRootImage, PreparedIndexRefreshBasis,
    PreparedManagedIndexSourceFork, PreparedRelationalCandidateAdmissionStop,
    PreparedRelationalPublication, PreparedRelationalPublicationAccelerators,
    PublicationPreparationDenial,
};
pub use publication::{
    RelationalRequiredOutputPublication, RelationalRequiredOutputRouteAbsence,
    RequiredOutputProducingImplementation, RequiredOutputPublication,
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
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
    RelationalTransactionStagingDenial, RelationalTransactionWriteLocus,
};
pub(crate) use validation::RelationalTransactionValidationInput;
pub use validation::{
    CustomInvariantExecutionReceipt, CustomInvariantNonApplicability,
    RelationalMutationInvariantEvidence, RelationalMutationProposalIdentity,
    ValidatedEntityStructTouch, ValidatedMutationFootprint, ValidatedMutationFootprintNotRequested,
    ValidatedMutationFootprintProjection, ValidatedMutationFootprintWork, ValidatedMutationTouch,
    ValidatedMutationTouchProjectionError, ValidatedMutationTouchProjectionWork,
    ValidatedMutationTouches, ValidatedRelationalProposal,
};

#[cfg(test)]
pub(crate) use publication::{native_birth_fixture, native_meaning_fixture};

#[cfg(test)]
pub(crate) use publication::native_roles_fixture;

pub(crate) use validation::ValidatedProposalState;
