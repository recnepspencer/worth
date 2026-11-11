//! Recorded transport meaning uses the same serialized original-publication driver.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCommittedDispatchOutboxObservation, WorthQueryPrimaryGraphApplicationRuntime,
};

impl QueryInboundJournalAdmission<'_> {
    pub(in crate::domain_computation) fn record_transport_binding<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<QueryTransportBinding, QueryDispatchJournalDenial> {
        let prior = self.journal.state().as_ref().and_then(|state| {
            state
                .ledger
                .inbound
                .transport_binding(owner.record().correlation().digest())
                .cloned()
        });
        let binding =
            runtime.original_transport_binding_record(self.journal, owner, prior.as_ref())?;
        if prior.as_ref() != Some(&binding) {
            let medium = self.journal.installed_medium()?;
            self.driver.publish(
                medium.as_ref(),
                QueryDispatchRecordIntent::Inbound(QueryInboundRecord::TransportBinding(
                    binding.clone(),
                )),
                request,
            )?;
        }
        Ok(binding)
    }
}

impl WorthQueryDispatchJournal {
    pub(in crate::domain_computation) fn recorded_transport_binding(
        &self,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<Option<QueryTransportBinding>, QueryDispatchJournalDenial> {
        let binding = self.state().as_ref().and_then(|state| {
            state
                .ledger
                .inbound
                .transport_binding(owner.record().correlation().digest())
                .cloned()
        });
        if let Some(binding) = &binding {
            if QueryInboundDispatchSource::from_owner(owner)? != binding.source
                || owner.record().operation_slot() != Some(binding.operation.as_str())
            {
                return Err(QueryDispatchJournalDenial::MalformedFrame);
            }
        }
        Ok(binding)
    }
    /// Read recorded completion DATA for this exact admitted original. This
    /// cannot create a live dispatch, causal phase, receipt or delivery.
    pub(in crate::domain_computation) fn recorded_transport_completion(
        &self,
        runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<crate::domain_computation::application_aftermath::external_effect::RecordedCompletionCausality, QueryDispatchJournalDenial>{
        let medium = self.installed_medium()?;
        if owner
            .committed_product_publication()
            .product_branch()
            .owner_identity()
            .durable()
            .map(|owner| owner.home())
            != Some(medium.home())
        {
            return Err(QueryDispatchJournalDenial::ForeignHome);
        }
        let custody = self
            .custody(owner.record().correlation().digest())
            .ok_or(QueryDispatchJournalDenial::UnknownAttempt)?;
        if custody.observation() != Some(QueryDispatchRecordedObservation::Completed) {
            return Err(QueryDispatchJournalDenial::MalformedFrame);
        }
        crate::domain_computation::application_aftermath::external_effect::RecordedCompletionCausality::describe(
            runtime, owner, custody.attempt()).map_err(|_| QueryDispatchJournalDenial::MalformedFrame)
    }
}
