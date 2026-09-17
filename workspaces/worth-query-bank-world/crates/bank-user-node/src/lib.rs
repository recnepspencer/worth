//! Independently authenticated user-node process boundary.

#![forbid(unsafe_code)]

mod configuration;
mod process;
mod protocol;
mod server;
mod session;

#[cfg(feature = "cold-certification")]
pub mod cold_certification;

pub use configuration::{
    BankUserNodeConfiguration, BankUserNodeConfigurationBuilder, BankUserNodeConfigurationError,
};
pub use process::run as run_bank_user_node_process;
pub use protocol::{
    BankUserNodeAccountActivityPageOutcome, BankUserNodeAccountActivityPageRequest,
    BankUserNodeAccountActivityResumeRequest, BankUserNodeAccountActivityStreamRequest,
    BankUserNodeAccountSummaryOutcome, BankUserNodeAccountSummaryRequest,
    BankUserNodeAuthorizationOutcome, BankUserNodeDenial, BankUserNodeDenialKind,
    BankUserNodeElevationApprovalOutcome, BankUserNodeElevationApprovalRequest,
    BankUserNodeElevationRequest, BankUserNodeElevationRequestOutcome,
    BankUserNodeElevationRevocationOutcome, BankUserNodeElevationRevocationRequest,
    BankUserNodeEstateDisbursementOutcome, BankUserNodeEstateDisbursementRequest,
    BankUserNodeEstateNotificationOutcome, BankUserNodeEstateNotificationRequest,
    BankUserNodeMandatoryReviewOutcome, BankUserNodeMandatoryReviewRequest,
    BankUserNodeMutationOutcome, BankUserNodeMutationRequest,
    BankUserNodeRecoveryInspectionOutcome, BankUserNodeRecoveryRequest,
    BankUserNodeRecoverySafeRetryOutcome,
};
pub use server::{BankUserNode, BankUserNodeBinding, BankUserNodeInstallError};
