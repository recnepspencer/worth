//! Abandoning an undelivered row returns its exact retained root consumption.
use super::super::{SourceCustody, SourceEpoch};

impl SourceCustody {
    /// Closing the sole delivery owner leaves no answer for this consumption.
    /// A semantic join retains the row's older key, but the bound source is
    /// the actual identity consumed by admission. Only that source is returned.
    pub(in super::super) fn return_undelivered_root(
        &mut self,
        identity: &SourceEpoch,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) {
        if self.retired.is_some() || self.completed || self.source.is_none() {
            return;
        }
        let Some(bound) = self.bound_sources.as_ref().and_then(|sources| {
            sources.iter().find(|source| {
                source.identity.same_semantic_source(identity) && source.scope == scope
            })
        }) else {
            return;
        };
        if self
            .retired_sources
            .iter()
            .any(|(retired, _)| retired == &bound.identity)
        {
            return;
        }
        self.consumed_sources
            .retain(|consumed| consumed != &bound.identity);
    }
}
