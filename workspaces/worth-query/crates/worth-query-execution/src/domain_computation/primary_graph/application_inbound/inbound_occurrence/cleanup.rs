//! Bounded host-requested turnover of settled inbound terminal custody.

use crate::domain_computation::primary_graph::application_inbound::InboundOccurrenceScopedOps;
use crate::domain_computation::primary_graph::application_inbound::ReconstructScopedOps;
use std::num::NonZeroUsize;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::receive::WorthQueryInboundAdmissionDenial as Denial;
use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};
use crate::domain_computation::application_aftermath::WorthQueryInboundCleanupReport;

pub(in crate::domain_computation::primary_graph) trait CleanupScopedOps<
    Schema: worth_query_installation::facade::ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn retained_accepted_for_cleanup_test(
        &self,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Option<
        std::sync::Arc<
            crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
        >,
    > {
        self.inbound()
            .custody()
            .lock()
            .unwrap()
            .accepted_by_correlation(correlation)
    }

    fn stale_publication_claim_for_cleanup_test(
        &self,
        accepted: &std::sync::Arc<
            crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence,
        >,
    ) -> (
        crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim,
        bool,
    ) {
        let mut custody = self.inbound().custody().lock().unwrap();
        (
            custody.claim_retryable_publication(accepted),
            custody.mark_publication_retryable(accepted),
        )
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> CleanupScopedOps<Schema> for HostRuntime {}

pub trait CleanupOps<Schema: worth_query_installation::facade::ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn cleanup_completed_inbound_occurrences(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_work: NonZeroUsize,
    ) -> Result<WorthQueryInboundCleanupReport, Denial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(Denial::ForeignVerifier)?;
        let sample = self
            .authorization_clock()
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
            .inbound()
            .custody()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .eligible_expired_terminal_correlations(handle.operation(), *now, work);
        if !correlations.is_empty() {
            self.verify_completed_inbound_for_cleanup(&correlations, *now)
                .map_err(|_| Denial::TerminalCleanupUnavailable)?;
        }
        let compact = self
            .inbound()
            .custody()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .eligible_expired_compact_terminals(
                handle.operation(),
                *now,
                work - correlations.len(),
            );
        for (correlation, original, completion) in &compact {
            let terminal = self
                .primary_provider()
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
            .inbound()
            .custody()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let settled = custody.cleanup_completed_exact(handle.operation(), *now, &correlations);
        let compact = custody.cleanup_compact_exact(handle.operation(), *now, &compact);
        Ok(settled.combine(compact))
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> CleanupOps<Schema> for HostRuntime {}

pub(crate) mod court_doors_for_test;
pub use court_doors_for_test::CleanupCleanupDoorsOps;
pub use court_doors_for_test::CleanupDoorsOps;
