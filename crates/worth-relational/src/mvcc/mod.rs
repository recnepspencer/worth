mod observation;
pub(crate) mod publication;
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
    CompanionBranchCell, CompanionBranchCellSlot, CompanionBranchImage, CompanionCellEditStop,
    CompanionDerivedImageRetention, CompanionDerivedRootAdmission, CompanionDerivedRootCleanup,
    CompanionDerivedRootCost, CompanionDerivedRootInstalled, CompanionDerivedRootPreparationStop,
    CompanionDerivedRootStopped, CompanionPreflightBudget, CompanionPreflightStop,
    CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
    DiscardedRelationalCommitCandidate, PendingCompanionRegistration, PerformedRelationalCommit,
    PreparedCompanionBranchCell, PreparedCompanionDerivedRoot, PreparedPublicationCompanionEffect,
    PreparedRelationalCommitCandidate, PublicationCompanionPreflight,
    PublicationCompanionRegistration, PublicationCompanionRegistrationPort,
    PublicationCompanionRegistrationStop, PublishRelationalCommit, RelationalPublicationCompanion,
    RelationalPublicationDeferred, RelationalPublicationDenial,
    RelationalPublicationDurabilityPosture, RelationalPublicationFailure,
    RelationalPublicationFailureKind, RelationalPublicationOutcome, RelationalPublicationPort,
    RelationalPublicationProjectionPosture, ReservedCompanionBranchCell,
    StaleRelationalBranchObservation,
};
pub(crate) use publication::{
    PreparedIndexRefreshBasis, PreparedRelationalCandidateAdmissionStop,
    PreparedRelationalPublication, PreparedRelationalPublicationAccelerators,
};
pub use publication::{
    RelationalRequiredOutputPublication, RelationalRequiredOutputRouteAbsence,
    RequiredOutputPublication, RequiredOutputPublicationMeaning, RequiredOutputRolePosture,
    RequiredOutputRoleTarget,
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
    RelationalTransactionIntent, RelationalTransactionReadLocus, RelationalTransactionRelationRead,
    RelationalTransactionRelationValue, RelationalTransactionStagingDenial,
    RelationalTransactionWriteLocus,
};
pub(crate) use validation::RelationalTransactionValidationInput;
pub use validation::{
    CustomInvariantExecutionReceipt, RelationalMutationInvariantEvidence,
    RelationalMutationProposalIdentity, ValidatedMutationFootprint,
    ValidatedMutationFootprintNotRequested, ValidatedMutationFootprintProjection,
    ValidatedMutationFootprintWork, ValidatedMutationTouch, ValidatedMutationTouchProjectionError,
    ValidatedMutationTouchProjectionWork, ValidatedMutationTouches, ValidatedRelationalProposal,
};

#[cfg(test)]
pub(crate) use publication::{native_birth_fixture, native_meaning_fixture};

#[cfg(test)]
pub(crate) use publication::native_roles_fixture;
