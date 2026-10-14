//! Bank transport adaptation.
//!
//! Authentik tokens and protocol types terminate in this crate.

#![forbid(unsafe_code)]

mod adapter;
mod authorization;
mod bank_identity;
mod client;
mod configuration;
mod credential;
mod error;
mod http;
mod scope;
mod validation;

#[path = "cold_certification.rs"]
pub mod cold_certification_for_test;
pub use adapter::AuthentikOidcAdapter;
pub use authorization::{
    AuthentikAuthorizationCallback, AuthentikAuthorizationRequest, AuthentikPendingAuthorization,
};
pub use bank_identity::AuthentikBankIdentity;
pub use configuration::{
    AuthentikOidcConfiguration, AuthentikOidcConfigurationBuilder, AuthentikOidcConfigurationError,
};
pub use credential::AuthentikOidcCredential;
pub use error::{
    AuthentikBankAuthenticationError, AuthentikBankIdentityBuildError,
    AuthentikOidcAdapterBuildError, AuthentikOidcFlowError,
};
pub use http::{
    run_bank_http_server_process, BankHttpAccountActivity, BankHttpAccountActivityEvent,
    BankHttpAccountActivityItem, BankHttpAccountActivityPageOutcome,
    BankHttpAccountActivityPageRequest, BankHttpAccountActivityResumeRequest,
    BankHttpAccountActivityStreamRequest, BankHttpAccountKind, BankHttpAccountStatus,
    BankHttpAccountSummary, BankHttpAccountSummaryOutcome, BankHttpAccountSummaryRequest,
    BankHttpCommitDescription, BankHttpCommitDisposition, BankHttpCredential, BankHttpDenial,
    BankHttpDenialKind, BankHttpEffectControls, BankHttpElevationApprovalOutcome,
    BankHttpElevationApprovalRequest, BankHttpElevationClosureKind, BankHttpElevationRequest,
    BankHttpElevationRequestOutcome, BankHttpElevationRevocationOutcome,
    BankHttpElevationRevocationRequest, BankHttpEmergencyAccessReason,
    BankHttpEstateDisbursementOutcome, BankHttpEstateDisbursementRequest,
    BankHttpEstateNotificationOutcome, BankHttpEstateNotificationRequest,
    BankHttpExpectedAccountStatus, BankHttpMandatoryReviewOutcome, BankHttpMandatoryReviewRequest,
    BankHttpMutationContinuation, BankHttpMutationControls, BankHttpMutationOperation,
    BankHttpMutationOutcome, BankHttpMutationPosture, BankHttpMutationReadControls,
    BankHttpMutationRequest, BankHttpNextAction, BankHttpPostingPurpose, BankHttpPreparedListener,
    BankHttpProcessAccount, BankHttpProcessAccountStatus, BankHttpProcessColdCertification,
    BankHttpProcessConfiguration, BankHttpProcessConfigurationError,
    BankHttpProcessEstateAftermathWorld, BankHttpProcessEstateElevationWorld,
    BankHttpProcessEstateWorld, BankHttpProcessOidcConfiguration, BankHttpProcessParticipant,
    BankHttpProcessWorld, BankHttpProtocolVersion, BankHttpQueryBasis, BankHttpQueryBasisPosture,
    BankHttpQueryCapabilityPurpose, BankHttpQueryDisclosure, BankHttpQueryDisclosurePosture,
    BankHttpQueryOmissionPosture, BankHttpQueryPublication, BankHttpRecoveryInspectionOutcome,
    BankHttpRecoveryPosture, BankHttpRecoveryRequest, BankHttpRecoveryRetryDisposition,
    BankHttpRecoverySafeRetryOutcome, BankHttpRecoveryStatus, BankHttpRecoveryWork,
    BankHttpRequestControls, BankHttpRestrictedBankField, BankHttpServer, BankHttpServerBinding,
    BankHttpServerClose, BankHttpServerCloseFailure, BankHttpServerConfiguration,
    BankHttpServerInstallation, BankRailCallbackServer, BankRailCallbackServerBinding,
    BankRailCloseAssessment, BankRailCompletionClose, BankRailCompletionCloseFailure,
    BankRailCompletionContinuation, BankRailCompletionServerInstallation,
};
