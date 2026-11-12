//! Original outstanding counts and recorded completed transport observations readmit together.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation;
use crate::domain_computation::primary_graph::InstalledTransportCompletion;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn readmit_dispatch_completion_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let originals = self
            .runtime
            .primary_provider
            .readmit_outstanding_dispatches(
                execution,
                &self.runtime.product_runtime.owner.observation_port(),
                &self.runtime.product_runtime.owner.inspection_port(),
            )
            .map_err(Denial::OwnerReadDenied)?;
        for owner in originals {
            let correlation = owner.record().correlation();
            if self.has_retained_installed_transport_completion(correlation) {
                continue;
            }
            if self
                .runtime
                .dispatch_journal
                .custody(correlation.digest())
                .and_then(|c| c.observation())
                == Some(QueryDispatchRecordedObservation::Completed)
            {
                let evidence = InstalledTransportCompletion::from_recorded_observation(
                    &self.runtime.dispatch_journal,
                    self.runtime.authority_identity(),
                    owner.clone(),
                )
                .map_err(|_| Denial::UnsupportedOutbox)?;
                let lease = self
                    .runtime
                    .primary_provider
                    .begin_external_dispatch_in_flight(&owner)
                    .map_err(|_| Denial::RetryBeforeAcceptance)?
                    .ok_or(Denial::OriginalDispatchHasNoInboundSupport)?;
                self.record_installed_transport_completion(evidence, lease)
                    .map_err(|_| Denial::RetryBeforeAcceptance)?;
            }
        }
        let driver = self
            .runtime
            .dispatch_journal
            .begin_inbound_admission(execution)
            .map_err(|_| Denial::RetryBeforeAcceptance)?;
        self.synchronize_durable_inbound(&driver, execution)
    }
}
