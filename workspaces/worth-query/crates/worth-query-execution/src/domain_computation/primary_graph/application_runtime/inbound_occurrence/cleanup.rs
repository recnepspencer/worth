//! Bounded host-requested turnover of settled inbound terminal custody.

use std::num::NonZeroUsize;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::receive::WorthQueryInboundAdmissionDenial as Denial;
use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};
use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn retained_accepted_for_cleanup_test(
        &self,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Option<
        std::sync::Arc<
            crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
        >,
    > {
        self.inbound_custody
            .lock()
            .unwrap()
            .accepted_by_correlation(correlation)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn stale_publication_claim_for_cleanup_test(
        &self,
        accepted: &std::sync::Arc<
            crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
        >,
    ) -> (
        crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim,
        bool,
    ) {
        let mut custody = self.inbound_custody.lock().unwrap();
        (
            custody.claim_retryable_publication(accepted),
            custody.mark_publication_retryable(accepted),
        )
    }

    /// Reclaim expired signed custody after rechecking its canonical World
    /// terminal pair. Transport terminals retain their own canonical truth;
    /// only their compact accepted-message meaning expires here.
    pub fn cleanup_completed_inbound_occurrences(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_work: NonZeroUsize,
    ) -> Result<WorthQueryInboundCleanupReport, Denial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(Denial::ForeignVerifier)?;
        let sample = self
            .authorization_clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
            .map_err(|_| Denial::TimeUnavailable)?;
        let AspectValue::UInt64(now) = sample.value() else {
            return Err(Denial::TimeUnavailable);
        };
        let limits = installed.contract.limits();
        let work = maximum_work
            .get()
            .min(usize::try_from(limits.maximum_cleanup_work.get()).unwrap_or(usize::MAX));
        let correlations = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .eligible_expired_terminal_correlations(handle.operation(), *now, work);
        if !correlations.is_empty() {
            self.verify_completed_inbound_for_cleanup(&correlations, *now)
                .map_err(|_| Denial::TerminalCleanupUnavailable)?;
        }
        let compact = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .eligible_expired_compact_terminals(
                handle.operation(),
                *now,
                work - correlations.len(),
            );
        for (correlation, original, completion) in &compact {
            let terminal = self
                .primary_provider
                .lookup_completed_inbound(correlation)
                .map_err(|_| Denial::TerminalCleanupUnavailable)?
                .ok_or(Denial::TerminalCleanupUnavailable)?;
            if terminal.operation() != handle.operation()
                || terminal.original_world_commit() != original
                || terminal.completion_world_commit() != completion
            {
                return Err(Denial::TerminalCleanupUnavailable);
            }
        }
        let mut custody = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let settled = custody.cleanup_completed_exact(handle.operation(), *now, &correlations);
        let compact = custody.cleanup_compact_exact(handle.operation(), *now, &compact);
        Ok(settled.combine(compact))
    }
}
