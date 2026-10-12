//! Abandoning an undelivered row returns its exact retained root consumption.
use super::super::{SourceCustody, SourceEpoch};

impl SourceCustody {
    /// Closing the sole delivery owner leaves no answer for this consumption.
    /// Only the exact still-retained source can issue its own root again.
    pub(in super::super) fn return_undelivered_root(
        &mut self,
        identity: &SourceEpoch,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) {
        if self.retired.is_none()
            && !self.completed
            && self.source.is_some()
            && !self
                .retired_sources
                .iter()
                .any(|(retired, _)| retired == identity)
            && self.bound_sources.as_ref().is_some_and(|sources| {
                sources
                    .iter()
                    .any(|source| &source.identity == identity && source.scope == scope)
            })
        {
            self.consumed_sources
                .retain(|consumed| consumed != identity);
        }
    }
}
