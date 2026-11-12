//! The evidence owner's recovered correlation candidates carry no live receipt.
use super::super::super::committed_dispatch_outbox::recovered::RecoveredDispatchSelector;
use super::*;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;

impl WorthQueryCompletedCommitEvidenceStore {
    pub(in crate::domain_computation::primary_graph::provider) fn next_recovered_dispatch(
        &self,
        after: Option<&ExternalEffectCorrelationIdentity>,
    ) -> Option<(ExternalEffectCorrelationIdentity, RecoveredDispatchSelector)> {
        use std::ops::Bound::{Excluded, Unbounded};
        let lower = after.map_or(Unbounded, Excluded);
        self.recovered_dispatch
            .range((lower, Unbounded))
            .next()
            .map(|(correlation, source)| (*correlation, source.clone()))
    }
    pub(in crate::domain_computation::primary_graph::provider) fn readmit_dispatch_correlations(
        &mut self,
        sources: BTreeMap<ExternalEffectCorrelationIdentity, RecoveredDispatchSelector>,
    ) -> Result<(), ()> {
        if sources
            .keys()
            .any(|key| self.by_dispatch_correlation.contains_key(key))
        {
            return Err(());
        }
        if !self.recovered_dispatch.is_empty()
            && (sources.len() != self.recovered_dispatch.len()
                || sources.iter().any(|(key, source)| {
                    self.recovered_dispatch
                        .get(key)
                        .is_none_or(|old| !old.same_source(source))
                }))
        {
            return Err(());
        }
        self.recovered_dispatch = sources;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::provider) fn recovered_dispatch_correlation(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<Option<RecoveredDispatchSelector>, ()> {
        if self.recovered_dispatch.contains_key(correlation)
            && self.by_dispatch_correlation.contains_key(correlation)
        {
            return Err(());
        }
        Ok(self.recovered_dispatch.get(correlation).cloned())
    }
}
