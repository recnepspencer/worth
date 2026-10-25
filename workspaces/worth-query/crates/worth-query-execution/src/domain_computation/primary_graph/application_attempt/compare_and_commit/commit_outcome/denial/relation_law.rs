//! The same original law fields survive either candidate refusal path.
use super::{denial_cause::DenialCause, WorthQueryApplicationCommitDenial};
use crate::domain_computation::WorthQueryRelationLawDenial;

impl WorthQueryApplicationCommitDenial {
    /// The native relation law and closing-edge identities, when present.
    /// The failed candidate is never evaluated again to recover this evidence.
    pub fn relation_law(&self) -> Option<WorthQueryRelationLawDenial<'_>> {
        match self.cause.as_deref()? {
            DenialCause::InvariantExecution(failure) => failure.relation_law(),
            DenialCause::ProviderSession(failure) => failure
                .native_preparation_error()
                .and_then(WorthQueryRelationLawDenial::from_commit_error),
            _ => None,
        }
    }
}
