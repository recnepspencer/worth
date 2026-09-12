//! Authoritative bank runtime composition.
//!
//! Transport and Authentik protocol details belong to downstream adapters.

#![forbid(unsafe_code)]

mod application_query;
mod authenticated_principal;
mod authentication_boundary;
mod bank_projection;
mod committed_dispatch_outbox;
mod domain_package;
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
mod operation_proposals;
mod ordinary;
mod principal_seed;
mod world_seed;

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
    BankApplicationPreviewSessionDenialKind, BankApplicationProjectionDenialKind,
    BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankApplicationQueryInstallationDenialKind, BankApplicationQueryLaneDenial,
    BankApplicationQueryParameterDenialKind, BankEstateEmergencyAccessActivityContinuation,
    BankEstateEmergencyAccessActivityLiveLease, BankEstateEmergencyAccessActivityLiveOutcome,
    BankEstateEmergencyAccessActivityLiveUpdate, BankEstateEmergencyAccessActivityPageResult,
    BankEstateEmergencyAccessActivityResult, BankEstateEmergencyAccountDetailsResult,
    BankGraphReadPlanReviewDenialKind, BankPreviewSession, BankPreviewSessionDiscardReceipt,
    BankProductSelectionDenialKind,
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
    BankInvariantProjectionTraversalDenial, BankRecoveryDenial, BankRecoveryDenialKind,
    BankRecoveryDurability, BankRecoveryExpiryDecision, BankRecoveryExpiryEvaluation,
    BankRecoveryIdempotencyResolution, BankRecoveryInspection, BankRecoveryPosture,
    BankRecoverySafeRetryReceipt, BankRecoverySupportTruth, BankRecoveryTransitionReceipt,
    BankRequestedEstateElevation, BankReviewedEstateElevation,
};
pub use external_effect_transport::BankExternalEffectTransportDenial;
pub use identity_runtime::{BankAuthenticationConfiguration, BankIdentityRuntime};
pub use operation_admission::{BankAdmittedOperation, BankOperationAdmissionError};
pub use operation_commit::{
    BankApplicationAttemptDenialKind, BankCommitCanonicalWorkEvidence,
    BankCommitCanonicalWorkPhases, BankCommitDenialKind, BankCommitDenialStage,
    BankCommitPreparationDenial, BankCommitReceipt, BankCommitRecoveryKind,
    BankMutationCommitOutcome, BankProviderFailureKind, BankProviderFailureStage,
    BankUnresolvedCommitEvidence,
};
pub use operation_proposals::{
    BankAuthorizedProposal, BankOperationProposalError, BankOperationProposals,
    BankSendMoneyPreparation,
};
pub use ordinary::{
    mutations, queries, BankApprovePendingPayment, BankAuthorizationDenial,
    BankAuthorizationDenialKind, BankEntityResolutionDenial, BankEntityResolutionDenialKind,
    BankIdempotencyResolutionDenialKind, BankMutation, BankMutationControls, BankMutationDenial,
    BankMutationExplanation, BankMutationExplanationStage, BankMutationForPrincipal,
    BankMutationMetadata, BankMutationOutcome, BankMutationProjectionWork,
    BankMutationProposalDenial, BankMutationStatus, BankOperationInstallationDenial,
    BankOperationInstallationDenialKind, BankPaymentContinuationDenial,
    BankPaymentInitiationOutcome, BankPendingPaymentContinuation, BankQuery, BankQueryForPrincipal,
    BankReadControlDenial, BankReadControls, BankReadyMutation, BankReadyQuery,
    BankRejectPendingPayment,
};
pub use principal_seed::BankPrincipalSeed;
pub use world_seed::{BankBusinessOwnerSeed, BankEmployeeAssignmentSeed, BankWorldSeed};
