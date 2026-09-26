//! Authoritative bank runtime composition.
//!
//! Transport and Authentik protocol details belong to downstream adapters.

#![forbid(unsafe_code)]

mod application_definition;
mod application_query;
mod approval_authentication;
mod authenticated_principal;
mod authentication_boundary;
mod bank_projection;
mod committed_dispatch_outbox;
mod error;
#[cfg(test)]
mod estate_capability_admission;
mod estate_progression;
mod external_effect_transport;
mod graph_bootstrap;
mod identity_runtime;
mod mutation_handlers;
mod operation_admission;
mod operation_commit;
mod ordinary;
mod principal_seed;
mod program_adoption;
mod world_seed;

pub use application_definition::{
    approved_business_payment_definition, ApprovedBusinessPaymentDefinitionDenial, BankApplication,
    BankApplicationP1,
};
pub use application_query::{
    BankAccountActivityContinuation, BankAccountActivityHistoricalResult,
    BankAccountActivityLiveLease, BankAccountActivityLiveOutcome, BankAccountActivityLiveUpdate,
    BankAccountActivityPageResult, BankAccountActivityQueryResult, BankAccountActivityRequest,
    BankAccountActivityRequestForPrincipal, BankAdmittedEstateEmergencyAccessActivityContinuation,
    BankAdmittedEstateEmergencyAccessActivityHistorical,
    BankAdmittedEstateEmergencyAccessActivityPreview,
    BankAdmittedEstateEmergencyAccountDetailsHistorical,
    BankAdmittedEstateEmergencyAccountDetailsPreview,
    BankApplicationCapabilityInstallationDenialKind, BankApplicationContinuationDenialKind,
    BankApplicationLiveCauseDenial, BankApplicationLiveCloseOutcome,
    BankApplicationLiveOpenDenialKind, BankApplicationLiveOverflow,
    BankApplicationLiveProjectionDenial, BankApplicationOneShotDenialKind,
    BankApplicationOutputSettlementDenialKind, BankApplicationPreviewSessionDenialKind,
    BankApplicationProjectionDenialKind, BankApplicationQueryAdmissionDenialKind,
    BankApplicationQueryDenial, BankApplicationQueryInstallationDenialKind,
    BankApplicationQueryLaneDenial, BankApplicationQueryParameterDenialKind,
    BankEstateEmergencyAccessActivityContinuation, BankEstateEmergencyAccessActivityLiveLease,
    BankEstateEmergencyAccessActivityLiveOutcome, BankEstateEmergencyAccessActivityLiveUpdate,
    BankEstateEmergencyAccessActivityPageResult, BankEstateEmergencyAccessActivityResult,
    BankEstateEmergencyAccountDetailsResult, BankGraphReadPlanReviewDenialKind, BankPreviewSession,
    BankPreviewSessionDiscardReceipt, BankProductSelectionDenialKind,
};
pub use approval_authentication::{
    BankApprovalAuthenticationConfiguration, BankApprovalCredential,
};
pub use approved_payment_workflow::{
    BankApprovedPaymentApplyOutcome, BankApprovedPaymentAssessment,
    BankApprovedPaymentPerformedOperation, BankApprovedPaymentPreparedRecovery,
    BankApprovedPaymentWorkflow, BankApprovedPaymentWorkflowError,
};
pub use authenticated_principal::BankAuthenticatedPrincipal;
pub use authentication_boundary::BankAuthenticationBoundary;
pub use bank_projection::{BankInvariantAggregateDenialKind, BankProjectionDenial};
pub use committed_dispatch_outbox::{
    BankCommittedDispatchOutboxObservation, BankCommittedDispatchOutboxReadDenial,
};
pub use error::{
    BankAuthenticationBoundaryBuildError, BankIdentityRuntimeBuildError,
    BankPrincipalAdmissionError, BankWorldSeedDenial,
};
pub use estate_progression::{
    BankApprovedEstateElevation, BankCapabilityDelegationProjectionDenial,
    BankCapabilityRevocationProjectionDenial, BankCommitRecoveryHandle,
    BankDeathNotificationProjectionDenial, BankEstateCaseOpeningProjectionDenial,
    BankEstateDisbursementProjectionDenial, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateElevationClosureKind,
    BankEstateElevationRequestOutcome, BankEstateElevationRetentionWork,
    BankEstateFreezeProjectionDenial, BankEstateIdempotencyResolutionDenial,
    BankEstateLifecycleProjectionDenial, BankEstateMandatoryReview,
    BankEstateMandatoryReviewOutcome, BankEstateOperationProjectionDenial,
    BankEstateProgressionDenial, BankEstateReleaseProjectionDenial,
    BankExecutorRecognitionProjectionDenial, BankInvariantDecisionPlanDenial,
    BankInvariantProjectionTraversalDenial, BankRecoveryClaimStatus, BankRecoveryDenial,
    BankRecoveryDenialKind, BankRecoveryDurability, BankRecoveryExpiryDecision,
    BankRecoveryExpiryEvaluation, BankRecoveryIdempotencyResolution, BankRecoveryInspection,
    BankRecoveryPosture, BankRecoverySafeRetryDenial, BankRecoverySafeRetryReceipt,
    BankRecoverySupportTruth, BankRecoveryTransitionReceipt, BankRequestedEstateElevation,
    BankReviewedEstateElevation,
};
pub use external_effect_transport::BankExternalEffectTransportDenial;
pub use identity_runtime::{BankAuthenticationConfiguration, BankIdentityRuntime};
pub use operation_commit::{
    BankApplicationAttemptDenialKind, BankCommitCanonicalWorkEvidence,
    BankCommitCanonicalWorkPhases, BankCommitDenialKind, BankCommitDenialStage,
    BankCommitPreparationDenial, BankCommitReceipt, BankCommitRecoveryKind,
    BankMutationCommitOutcome, BankProviderFailureKind, BankProviderFailureStage,
    BankUnresolvedCommitEvidence,
};
pub use ordinary::{
    mutations, queries, BankAccountAccessExecution, BankAuthorizationDenial,
    BankAuthorizationDenialKind, BankBusinessAccountCreationExecution, BankEntityResolutionDenial,
    BankEntityResolutionDenialKind, BankMoneyMovementExecution, BankMutation, BankMutationControls,
    BankMutationForPrincipal, BankOperationInstallationDenial, BankOperationInstallationDenialKind,
    BankPaymentContinuationDenial, BankPaymentDecisionExecution, BankPaymentInitiationOutcome,
    BankPendingPaymentContinuation, BankPersonalAccountCreationExecution,
    BankProgramMutationExecution, BankQuery, BankQueryForPrincipal, BankReadControlDenial,
    BankReadControls, BankReadyMutation, BankReadyQuery, BankRejectPendingPayment,
};
pub use principal_seed::BankPrincipalSeed;
pub use program_adoption::{BankProgramAdoptionPreparationDenial, BankProgramInspectionDenial};
pub use world_seed::{BankBusinessOwnerSeed, BankEmployeeAssignmentSeed, BankWorldSeed};
mod approved_payment_workflow;
