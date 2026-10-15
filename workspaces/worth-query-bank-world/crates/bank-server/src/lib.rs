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
mod inbound_completion;
mod mutation_handlers;
mod operation_admission;
mod operation_commit;
mod ordinary;
mod principal_seed;
mod program_adoption;
mod program_deployment_grant_seed;
mod program_deployment_revocation;
mod world_seed;

pub use application_definition::{
    approved_business_payment_definition, ApprovedBusinessPaymentDefinitionDenial, BankApplication,
    BankApplicationP1, BankApplicationP2, DepositAction, DepositDelivery, OpeningFundingAction,
    OpeningFundingDelivery, ReverseJournalAction, ReverseJournalDelivery, SendMoneyAction,
    SendMoneyDelivery, WithdrawalAction, WithdrawalDelivery, APPROVAL_LIMIT,
    APPROVAL_LIMIT_OPERAND,
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
    BankApprovedPaymentAssessmentDemand, BankApprovedPaymentAssessmentProgress,
    BankApprovedPaymentCancellation, BankApprovedPaymentPerformedOperation,
    BankApprovedPaymentPreparedRecovery, BankApprovedPaymentWorkflow,
    BankApprovedPaymentWorkflowError, BankPaymentAssessmentSettlement,
};
pub use authenticated_principal::BankAuthenticatedPrincipal;
pub use authentication_boundary::BankAuthenticationBoundary;
pub use bank_projection::{BankInvariantAggregateDenialKind, BankProjectionDenial};
pub use committed_dispatch_outbox::{
    BankCommittedDispatchOutboxObservation, BankCommittedDispatchOutboxReadDenial,
    BankExternalDispatchAttemptDenial,
};
pub use error::{
    BankAuthenticationBoundaryBuildError, BankIdentityRuntimeBuildError,
    BankPrincipalAdmissionError, BankWorldSeedDenial,
};
pub use estate_progression::{
    BankApprovedEstateElevation, BankCapabilityDelegationProjectionDenial,
    BankCapabilityRevocationProjectionDenial, BankCommitRecoveryHandle,
    BankDeathNotificationProjectionDenial, BankDelegationCaptureFailure, BankDelegationCommand,
    BankDelegationPreparation, BankEstateCaseOpeningProjectionDenial,
    BankEstateDisbursementProjectionDenial, BankEstateElevationApprovalOutcome,
    BankEstateElevationCloseOutcome, BankEstateElevationClosureKind,
    BankEstateElevationRequestOutcome, BankEstateElevationRetentionWork,
    BankEstateFreezeProjectionDenial, BankEstateIdempotencyResolutionDenial,
    BankEstateLifecycleProjectionDenial, BankEstateMandatoryReview,
    BankEstateMandatoryReviewOutcome, BankEstateOperationProjectionDenial,
    BankEstateProgressionDenial, BankEstateReleaseProjectionDenial,
    BankExecutorRecognitionProjectionDenial, BankInvariantDecisionPlanDenial,
    BankInvariantProjectionTraversalDenial, BankRecoveryClaimStatus, BankRecoveryDenial,
    BankRecoveryDenialKind, BankRecoveryDisposalDenial, BankRecoveryDisposalReceipt,
    BankRecoveryDurability, BankRecoveryExpiryDecision, BankRecoveryExpiryEvaluation,
    BankRecoveryIdempotencyResolution, BankRecoveryInspection, BankRecoveryPosture,
    BankRecoverySafeRetryDenial, BankRecoverySafeRetryReceipt, BankRecoverySupportTruth,
    BankRecoveryTransitionReceipt, BankRequestedEstateElevation, BankReviewedEstateElevation,
    BankRevocationCaptureFailure, BankRevocationCommand, BankRevocationPreparation,
};
pub use external_effect_transport::BankExternalEffectTransportDenial;
pub use identity_runtime::{BankAuthenticationConfiguration, BankIdentityRuntime};
pub use inbound_completion::{
    BankEstateRailCompletionInstallationDenial, BankEstateRailCompletionRoute,
    BankPaymentRailCompletionRoute,
};
pub use operation_commit::{
    BankApplicationAttemptDenialKind, BankCommitCanonicalWorkEvidence,
    BankCommitCanonicalWorkPhases, BankCommitDenialKind, BankCommitDenialStage,
    BankCommitPreparationDenial, BankCommitReceipt, BankCommitRecoveryKind,
    BankMutationCommitOutcome, BankProviderFailureKind, BankProviderFailureStage,
    BankUnresolvedCommitEvidence,
};
pub use ordinary::{
    mutations, queries, BankAuthorizationDenial, BankAuthorizationDenialKind,
    BankEntityResolutionDenial, BankEntityResolutionDenialKind, BankMutation, BankMutationControls,
    BankMutationForPrincipal, BankOperationConstructionRefusal, BankOperationInstallationDenial,
    BankOperationInstallationDenialKind, BankPaymentContinuationDenial,
    BankPendingPaymentContinuation, BankQuery, BankQueryForPrincipal, BankReadControlDenial,
    BankReadControls, BankReadyMutation, BankReadyQuery, BankRejectPendingPayment,
};
pub use principal_seed::BankPrincipalSeed;
pub use program_adoption::{BankProgramAdoptionPreparationDenial, BankProgramInspectionDenial};
pub use program_deployment_grant_seed::{
    BankProgramDeploymentGrantSeed, BankProgramDeploymentGrantSeedDenial,
};
pub use world_seed::{BankBusinessOwnerSeed, BankEmployeeAssignmentSeed, BankWorldSeed};
mod approved_payment_workflow;

#[cfg(test)]
mod test_host_policy;
#[cfg(test)]
extern crate self as bank_server;
#[cfg(test)]
#[allow(
    dead_code,
    reason = "shared integration fixture support is reused by the workflow request proof"
)]
#[path = "../tests/support/mod.rs"]
mod support;

pub use application_definition::{
    AccountAccessGrantAction, AccountAccessGrantDelivery, AccountAccessRevocationAction,
    AccountAccessRevocationDelivery, BusinessAccountCreationAction,
    BusinessAccountCreationDelivery, PaymentInitiationAction, PaymentInitiationDelivery,
    PaymentRejectionAction, PaymentRejectionDelivery, PersonalAccountCreationAction,
    PersonalAccountCreationDelivery,
};

pub use application_definition::{
    DeathNoticeRetransmissionAction, DeathNoticeRetransmissionDelivery, DeathNotificationAction,
    DeathNotificationDelivery, EstateAccountFreezeAction, EstateAccountFreezeDelivery,
    EstateCaseOpeningAction, EstateCaseOpeningDelivery, EstateDisbursementAction,
    EstateDisbursementDelivery, EstateReleaseAction, EstateReleaseDelivery,
    ExecutorRecognitionAction, ExecutorRecognitionDelivery, ProgramDeploymentRevocationAction,
    ProgramDeploymentRevocationDelivery,
};

pub use operation_commit::{BankManagedCleanup, BankReleasedManagedCleanup};

pub use application_definition::ApprovedPaymentEffectAction;
pub use approved_payment_workflow::BankAdvancedApprovedPaymentWorkflow;

pub use approved_payment_workflow::BankCompletedApprovedPayment;
