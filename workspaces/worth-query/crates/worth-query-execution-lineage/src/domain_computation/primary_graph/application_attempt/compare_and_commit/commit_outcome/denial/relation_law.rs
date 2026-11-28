//! The same original law fields survive either candidate refusal path.
use super::{denial_cause::DenialCause, WorthQueryApplicationCommitDenial};
use crate::domain_computation::InvariantViolationFields;

impl WorthQueryApplicationCommitDenial {
    /// The native relation law and original endpoint identities, when present.
    /// The failed candidate is never evaluated again to recover this evidence.
    pub fn relation_law(&self) -> Option<&InvariantViolationFields> {
        match self.cause.as_deref()? {
            DenialCause::InvariantExecution(failure) => failure.relation_law(),
            DenialCause::ProviderSession(failure) => failure
                .native_preparation_error()
                .and_then(worth_query_execution_contracts::facade::relational_invariant_fields)
                .filter(|fields| worth_query_execution_contracts::facade::is_relation_law(fields)),
            _ => None,
        }
    }
}
