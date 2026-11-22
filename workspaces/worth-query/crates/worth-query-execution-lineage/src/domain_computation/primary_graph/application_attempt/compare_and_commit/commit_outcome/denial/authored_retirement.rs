//! The owner refusal retains its actual record subjects through both admission paths.
use super::{denial_cause::DenialCause, WorthQueryApplicationCommitDenial};

impl WorthQueryApplicationCommitDenial {
    /// Original Native authored retirement refusal, including affected records.
    pub fn authored_retirement_error(
        &self,
    ) -> Option<&worth_relational::facade::publication::PublicationError> {
        match self.cause.as_deref()? {
            DenialCause::InvariantExecution(failure) => failure.authored_retirement_error(),
            DenialCause::ProviderSession(failure) => match failure.native_preparation_error()? {
                worth_relational::facade::mvcc::TransactionCommitError::Publication {
                    error,
                    ..
                } if error.authored_retirement_refusal().is_some() => Some(error),
                _ => None,
            },
            _ => None,
        }
    }
}
