//! Location-independent bootstrap completion; stored meaning grants no authority.
mod codec;
mod publication;
pub(in crate::domain_computation::primary_graph) mod readmission;
mod record;
pub(in crate::domain_computation::primary_graph) use publication::publish;

use super::super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
fn denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
        subject,
    )
}
fn recovery_denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}
fn request(cause: worth_execution::WorkCeilingDenial) -> WorthQueryPrimaryGraphInstallationDenial {
    super::request_denial(super::super::application_contribution::advancement_denial(
        cause,
    ))
}
