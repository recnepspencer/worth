//! The sole public aggregation surface for `worth-runtime-world`.
//!
//! This module contains exports only. Runtime behavior remains in the private
//! owner modules so callers cannot reach around the composition boundary.

// Public signatures below name the already-issued component owner bundles and
// tokens. Re-exporting those exact types here keeps the World facade complete
// without exposing a constructor or a second authority path.
pub use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisAdmissionIdentity,
    RelationalBranchBasisPort, RelationalBranchIdentity, RelationalOwnerServicePorts,
};

pub use worth_relational::facade::history::{RelationalCommitIdentity, RelationalCommitReceipt};

pub use worth_relational::facade::mvcc::RelationalTransactionIntent;

pub use worth_runtime_bridge::facade::{
    AdmittedRuntimeWorldCorrespondenceBasis, RuntimeWorldCorrespondencePort,
};

pub use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchAdvanceOutcome, SignalBranchBasisAdmissionIdentity,
    SignalBranchForkOutcome, SignalOwnerCancellationToken, SignalOwnerServicePorts,
};

pub use worth_signal::facade::{SignalError, SignalTransaction};

pub use crate::basis::AdmittedCompositeRuntimeWorldBasis;

pub use crate::branch::{
    ComponentBranchTarget, CustodyComponent, NoEffectRuntimeWorldBootstrap,
    OwnerCreatedComponentCustodyRecord, OwnerRetirementWork, PerformedRuntimeWorldBootstrap,
    ProductBranchCreationIntent, ProductBranchCreationPlans, ProductBranchName,
    ProductBranchNameDenial, ProductBranchObservation, ProductBranchObservationMismatch,
    ProductBranchObservationMismatchAxis, ProductBranchReferenceSnapshot,
    ProductBranchRetirementReport, RelationalBranchCreationPlan, RuntimeWorldBootstrapIntent,
    RuntimeWorldBootstrapNoEffectCause, RuntimeWorldBootstrapOutcome,
    RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchRetirementDenial,
    SignalBranchCreationPlan,
};

pub use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial, RuntimeWorldBudgetInstallation,
    RuntimeWorldBudgetLimit, RuntimeWorldBudgetResource, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};

pub use crate::history::{
    CompositeCallerCorrelation, CompositeCommitParent, CompositeCommitProvenance,
    CompositeComponentChangePosture, CompositeHistoryCatalogDenial,
    CompositeHistoryReclamationRequest, CompositeHistoryTraversal, CompositeRuntimeWorldCommit,
    CompositeSignalPublicationIdentity, HistoryCatalogCounters, HistoryMetadataLedger,
    HistoryReclamationDenial, HistoryReclamationOutcome, OrdinaryParent,
};

pub use crate::identity::{
    CompositeBasisKey, CompositeCommitIdentity, CompositePublicationAttemptIdentity,
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchReferenceGeneration,
    ProductUnpublishedOwnerEffectsIdentity, RuntimeWorldBootstrapAttemptIdentity,
    RuntimeWorldIdentityExhaustion, RuntimeWorldIdentityFamily, RuntimeWorldOwnerIdentity,
};

pub use crate::lifecycle::{
    MissingRuntimeWorldInput, RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchPort,
    RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldCloseDenial, RuntimeWorldCloseReport,
    RuntimeWorldInspectionPort, RuntimeWorldInstant, RuntimeWorldLifecyclePort,
    RuntimeWorldObservationPort, RuntimeWorldOwnedAsyncRequestAdmissionDenial,
    RuntimeWorldOwnedAsyncRevalidationDenial, RuntimeWorldOwner, RuntimeWorldOwnerBuilder,
    RuntimeWorldOwnerLifecycleObservation, RuntimeWorldOwnerUnavailable,
    RuntimeWorldPublicationPort, RuntimeWorldRecoveryPort, RuntimeWorldRetainedRecordReport,
    RuntimeWorldServiceDenial,
};

pub use crate::publication::{
    CompositeAttemptCancellationPosture, CompositeAttemptProgress, CompositeComponentIntent,
    CompositeLateCancellationPosture, CompositeOwnerExecutionResults,
    CompositePublicationCostCounters, CompositePublicationIntent, CompositePublicationOrder,
    CompositeRelationalOwnerResult, CompositeSignalOwnerResult, ConsumedCompositePublication,
    NoEffectCause, NoEffectCompositePublication, PerformedCompositePublication,
    PreparedCompositePublicationWithSignal, PreparedCompositePublicationWithoutSignal,
    RelationalAttemptProgress, RelationalAttemptProgressPosture, RelationalComponentPlan,
    RelationalComponentPlanPosture, RuntimeWorldCancellationSource, RuntimeWorldCancellationToken,
    RuntimeWorldConditionalDefinitionPublicationOutcome, RuntimeWorldPublicationOutcome,
    RuntimeWorldPublicationPhase, RuntimeWorldUnpublishedConditionalDefinition,
    SignalAttemptProgress, SignalAttemptProgressPosture, SignalComponentPlan,
    SignalComponentPlanPosture, WithSignal, WithoutSignal,
};

pub use crate::recovery::{
    PerformedPublicationRecoveryDenial, ProductUnpublishedCause, ProductUnpublishedNextAction,
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle,
    ProductUnpublishedRetentionPosture, RecoveryContinuationContract, RuntimeWorldRecoveryDenial,
};

pub use crate::inspection::{
    RuntimeWorldHistorySnapshot, RuntimeWorldRecoveryCosts, RuntimeWorldRecoveryCursor,
    RuntimeWorldRecoveryPage, RuntimeWorldRecoveryRecordState, RuntimeWorldRecoveryRow,
    RuntimeWorldRecoverySnapshot, RuntimeWorldRetentionEntry,
    RuntimeWorldRetentionInspectionDenial, RuntimeWorldRetentionKey, RuntimeWorldRetentionSnapshot,
};

pub use crate::retention::{
    ComponentBasisDependencyClass, ComponentBasisDependencyCounts, RetentionCostSnapshot,
    RetentionReclamationReport,
};

pub use worth_relational::facade::transactions::CommitResult;

#[cfg(feature = "test-operation-control")]
pub use crate::lifecycle::{RuntimeWorldOperationControl, RuntimeWorldProductComparePause};
