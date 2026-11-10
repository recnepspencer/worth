//! The sole public aggregation surface for `worth-runtime-world`.
//!
//! This module contains exports only. Runtime behavior remains in the private
//! owner modules so callers cannot reach around the composition boundary.
pub use crate::branch::{ObservedProductBranches, ProductBranchInventoryDenial};
pub use crate::lifecycle::{
    CapturedWorldInterimAdmission, CapturedWorldInterimOrigin, WorldInterimCheckpointPort,
    WorldInterimCheckpointRefusal,
};

pub use crate::durability::{
    PreparedWorldCheckpoint, PreparedWorldHomeData, PreparedWorldHomeSource,
    RuntimeWorldDurableMedium, WorldCheckpointJournalRecord, WorldCheckpointOriginRetirement,
    WorldCheckpointOriginRetirementDenial, WorldCheckpointPort, WorldCheckpointReadSource,
    WorldCheckpointReadView, WorldCheckpointSelection, WorldCheckpointSubject,
    WorldDurableArtifact, WorldDurableFrameDenial, WorldDurableHomeBinding,
    WorldDurableMediumDenial, WorldHomeProgramLineage, WorldOpenComponentIntent,
    WorldPublicationCompletion, WorldPublicationObservation, WorldPublicationTaskPosture,
    WorldReadmissionDenial, WORLD_ARTIFACT_PROTOCOL, WORLD_ARTIFACT_VERSION,
    WORLD_CHECKPOINT_PROTOCOL, WORLD_CHECKPOINT_VERSION,
};

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
    ComponentBranchTarget, CurrentProductHead, CustodyComponent, NoEffectRuntimeWorldBootstrap,
    OwnerCreatedComponentCustodyRecord, OwnerRetirementWork, PerformedRuntimeWorldBootstrap,
    PreparedProductExactHistoryPage, ProductBranchAncestorSelection,
    ProductBranchAncestorSelectionDenial, ProductBranchCreationIntent, ProductBranchCreationPlans,
    ProductBranchCurrentnessFailure, ProductBranchHistoryTraversal, ProductBranchName,
    ProductBranchNameDenial, ProductBranchObservation, ProductBranchObservationMismatch,
    ProductBranchObservationMismatchAxis, ProductBranchReferenceDescription,
    ProductBranchReferenceReadmissionDenial, ProductBranchReferenceSnapshot,
    ProductBranchRetirementReport, ProductExactHistoryCursor, ProductExactHistoryDenial,
    ProductExactHistoryPage, RecoveredRuntimeWorldRootAuthority, RelationalBranchCreationPlan,
    RetainedProductBranchCreationSource, RuntimeWorldBootstrapIntent,
    RuntimeWorldBootstrapNoEffectCause, RuntimeWorldBootstrapOutcome,
    RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchRetirementDenial,
    SignalBranchCreationPlan, WorldDurableBootstrapOutcome, WorldDurableBootstrapRefusal,
    PRODUCT_BRANCH_REFERENCE_DESCRIPTION_VERSION,
};

pub use crate::budget::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial, RuntimeWorldBudgetInstallation,
    RuntimeWorldBudgetLimit, RuntimeWorldBudgetResource, RuntimeWorldBudgets,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
};

pub use crate::history::{
    CompositeAncestorSelectionDenial, CompositeCallerCorrelation, CompositeCommitParent,
    CompositeCommitProvenance, CompositeComponentChangePosture, CompositeHistoryCatalogDenial,
    CompositeHistoryTraversal, CompositeRuntimeWorldCommit, CompositeSignalPublicationIdentity,
    HistoryCatalogCounters, HistoryMetadataLedger, HistoryReclamationDenial, OrdinaryParent,
    RetiredBranchHistoryRelease, RuntimeWorldPerformedPublicationProtection,
    RuntimeWorldPublicationCursor, RuntimeWorldPublicationFrontier, RuntimeWorldPublicationPage,
    RuntimeWorldPublicationRow, WorldObservedMemoryPublication, WorldReadmittedPublication,
    WorldReadmittedPublicationDenial,
};

pub use crate::identity::{
    CompositeBasisKey, CompositeCommitIdentity, CompositePublicationAttemptIdentity,
    EphemeralWorldOwnerIdentity, ProductBranchIdentity, ProductBranchIncarnation,
    ProductBranchReferenceGeneration, ProductUnpublishedOwnerEffectsIdentity,
    RuntimeWorldBootstrapAttemptIdentity, RuntimeWorldIdentityExhaustion,
    RuntimeWorldIdentityFamily, RuntimeWorldOwnerIdentity,
};
pub use crate::publication::{
    home_transition_caller_association_commitment, ReservedWorldHomeTransitionAssignments,
    WorldDurablePublicationDenial, WorldDurablePublicationOutcome,
    WorldHomeTransitionEffectControl, WorldHomeTransitionEffectDenial,
    WorldHomeTransitionEffectInterruption, WorldHomeTransitionEffectOutcome,
    WorldHomeTransitionEffectProgress, WorldHomeTransitionOpenResumeDenial,
    WorldHomeTransitionOpenResumeRefused, WorldHomeTransitionReservationRefusal,
    WorldHomeTransitionStepReservation, WorldHomeTransitionUnopenedDenial,
    WorldNativePreparationDenial, WorldNativePreparationRefusal, WorldOpenAttemptOutcome,
    WorldOpenAttemptRefusal, WorldOpenedPublicationAttempt, WorldPendingDurablePublication,
    WorldPendingOpenAttempt, WorldPreparedPublicationWithSignal,
    WorldPreparedPublicationWithoutSignal, WorldQueryNativePreparation,
    WorldQuerySignalPreparation,
};

pub use crate::lifecycle::{
    MissingRuntimeWorldInput, PreparedWorldHomeProgramBinding, PreparedWorldRelationalCutover,
    RuntimeWorldBranchCreationCompletion, RuntimeWorldBranchCreationNoEffect,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRelease, RuntimeWorldBranchPort,
    RuntimeWorldBuildDenial, RuntimeWorldClock, RuntimeWorldClockSource, RuntimeWorldCloseDenial,
    RuntimeWorldCloseReport, RuntimeWorldExecutionPlacement, RuntimeWorldHomeBranchCreationRequest,
    RuntimeWorldInspectionPort, RuntimeWorldInstant, RuntimeWorldLifecyclePort,
    RuntimeWorldObservationPort, RuntimeWorldOwnedAsyncManagedRevalidationDenial,
    RuntimeWorldOwnedAsyncRequestAdmissionDenial, RuntimeWorldOwnedAsyncRevalidationDenial,
    RuntimeWorldOwner, RuntimeWorldOwnerBuilder, RuntimeWorldOwnerLifecycleObservation,
    RuntimeWorldOwnerUnavailable, RuntimeWorldPreCloseAdmission, RuntimeWorldPublicationPort,
    RuntimeWorldRecoveredCustodyReport, RuntimeWorldRecoveryPort,
    RuntimeWorldResultSnapshotCleanup, RuntimeWorldResultSnapshotCleanupDenial,
    RuntimeWorldResultSnapshotCleanupProgress, RuntimeWorldResultSnapshotCleanupReport,
    RuntimeWorldRetainedRecordReport, RuntimeWorldServiceDenial, RuntimeWorldUnboundHomeBuilder,
    WorldHomeProgramBindingDenial, WorldSignalForkPreparationDenial,
};

pub use crate::publication::{
    CompositeAttemptCancellationPosture, CompositeAttemptProgress, CompositeComponentIntent,
    CompositeLateCancellationPosture, CompositeOwnerExecutionResults,
    CompositePublicationCostCounters, CompositePublicationIntent, CompositePublicationOrder,
    CompositeRelationalOwnerResult, CompositeSignalOwnerResult, ConsumedCompositePublication,
    CurrentProductReadDependencyDenial, NoEffectCause, NoEffectCompositePublication,
    PerformedCompositePublication, PlannedProductReferenceSuccessor,
    PreparedCompositePublicationWithSignal, PreparedCompositePublicationWithoutSignal,
    PreparedConditionalDefinitionRecovery, RelationalAttemptProgress,
    RelationalAttemptProgressPosture, RelationalComponentPlan, RelationalComponentPlanPosture,
    RuntimeWorldCancellationSource, RuntimeWorldCancellationToken,
    RuntimeWorldConditionalDefinitionPublicationOutcome, RuntimeWorldPublicationOutcome,
    RuntimeWorldPublicationPhase, RuntimeWorldUnpublishedConditionalDefinition,
    SignalAttemptProgress, SignalAttemptProgressPosture, SignalComponentPlan,
    SignalComponentPlanPosture, WithSignal, WithoutSignal,
};

pub use crate::recovery::{
    PerformedPublicationRecoveryDenial, ProductUnpublishedCause, ProductUnpublishedCleanup,
    ProductUnpublishedNextAction, ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle,
    ProductUnpublishedRetentionPosture, RecoveryContinuationContract, RuntimeWorldRecoveryDenial,
    RuntimeWorldSettledRelationalAdoptionDenial, WorldRecoveredContinuationDenial,
    WorldRecoveredContinuationOutcome, WorldRecoveredReleaseOutcome,
    WorldRecoveredUnpublishedEffects, WorldReleasedEffects, WorldReleasedOriginal,
    WorldReleasedOriginalPage,
};

pub use crate::inspection::{
    RuntimeWorldHistorySnapshot, RuntimeWorldRecoveryCosts, RuntimeWorldRecoveryCursor,
    RuntimeWorldRecoveryPage, RuntimeWorldRecoveryRecordState, RuntimeWorldRecoveryRow,
    RuntimeWorldRecoverySnapshot, RuntimeWorldRetentionEntry,
    RuntimeWorldRetentionInspectionDenial, RuntimeWorldRetentionKey, RuntimeWorldRetentionSnapshot,
};

pub use crate::retention::{
    ComponentBasisDependencyClass, ComponentBasisDependencyCounts, RetentionCostSnapshot,
    RetentionObligationDenial, RetentionReclamationReport, RetentionTransferDenial,
};

pub use worth_relational::facade::transactions::CommitResult;

pub use crate::lifecycle::{
    RuntimeWorldOperationControlForTest, RuntimeWorldPerformedPublicationObservationForTest,
    RuntimeWorldProductComparePauseForTest, RuntimeWorldProductCurrentnessPauseForTest,
    RuntimeWorldRecoveredDispositionObservationForTest,
};

pub use crate::lifecycle::{
    world_branch_reads_on_this_thread_for_test, RuntimeWorldReadObservation,
};

pub use crate::publication::{
    ConsumedPublicationRetirement, ConsumedPublicationRetirementDenial,
    ConsumedPublicationRetirementProgress, PreparedConsumedPublicationRetirement,
    RetiredConsumedPublication,
};

pub use crate::lifecycle::{
    RetiredRuntimeWorldHome, RuntimeWorldHomeRetirement, RuntimeWorldHomeRetirementDenial,
    RuntimeWorldHomeRetirementProgress,
};

pub use crate::lifecycle::{
    RuntimeWorldAuthoredSignalAttachment, RuntimeWorldAuthoredSignalAttachmentDenial,
};

pub use crate::lifecycle::RuntimeWorldAuthoredHomeBuilder;

pub use crate::publication::world_branch_program_source_commitment;
