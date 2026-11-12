//! Preserve the original selected transport audience across fresh verifier configuration.
use super::transport_binding::WorthQueryTransportCompletionBindingDenial;
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryDispatchJournalDenial as JournalDenial, QueryInboundDispatchSource, QueryTransportBinding,
    WorthQueryDispatchJournal,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn original_transport_binding_record(
        &self,
        journal: &WorthQueryDispatchJournal,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        prior: Option<&QueryTransportBinding>,
    ) -> Result<QueryTransportBinding, JournalDenial> {
        if !std::ptr::eq(self.runtime.dispatch_journal.as_ref(), journal)
            || owner.relational_runtime_instance_id()
                != self
                    .runtime
                    .product_runtime
                    .source()
                    .authoritative_source_profile()
                    .runtime_instance_id()
            || owner
                .committed_product_publication()
                .product_branch()
                .owner_identity()
                != self.runtime.product_runtime.owner.owner_identity()
        {
            return Err(JournalDenial::ForeignHome);
        }
        let record = owner.record();
        let contract = record.inbound().ok_or(JournalDenial::MalformedFrame)?;
        let operation = record
            .operation_slot()
            .ok_or(JournalDenial::MalformedFrame)?;
        let source = QueryInboundDispatchSource::from_owner(owner)?;
        if let Some(prior) = prior {
            if prior.operation != operation || prior.source != source {
                return Err(JournalDenial::MalformedFrame);
            }
            if prior.audience.is_some() {
                return Ok(prior.clone());
            }
        }
        let installed = self
            .runtime
            .inbound_verifiers
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let audience = match installed.get(operation) {
            Some(verifier) if verifier.contract == *contract => {
                Some(verifier.verifier.audience().to_owned())
            }
            Some(_) => return Err(JournalDenial::MalformedFrame),
            None => None,
        };
        Ok(QueryTransportBinding {
            correlation: *record.correlation().digest(),
            operation: operation.to_owned(),
            audience,
            source,
        })
    }

    pub(in crate::domain_computation) fn record_original_transport_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<Option<QueryTransportBinding>, JournalDenial> {
        if owner.record().inbound().is_none() || !self.runtime.dispatch_journal.is_durable() {
            return Ok(None);
        }
        let driver = self
            .runtime
            .dispatch_journal
            .begin_inbound_admission(execution)?;
        driver
            .record_transport_binding(self, owner, execution)
            .map(Some)
    }

    pub(in crate::domain_computation) fn admit_installed_transport_completion_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<
        WorthQueryInstalledTransportCompletionBinding,
        WorthQueryTransportCompletionBindingDenial,
    > {
        use WorthQueryTransportCompletionBindingDenial as Denial;
        let Some(binding) = self
            .record_original_transport_binding(execution, owner)
            .map_err(|_| Denial::RecordedBindingUnavailable)?
        else {
            return self.resolve_installed_transport_completion_binding(owner);
        };
        Ok(WorthQueryInstalledTransportCompletionBinding {
            operation: binding.operation,
            audience: binding.audience.ok_or(Denial::VerifierMissing)?,
            contract: owner
                .record()
                .inbound()
                .ok_or(Denial::OutboxWithoutInboundContract)?
                .clone(),
            runtime: self.runtime.authority_identity(),
        })
    }
}
