//! Restore immutable accepted obligations by rereading their actual original source.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryInboundAcceptedMessage, QueryInboundDispatchSource, QueryInboundMessageKey,
    QueryInboundRecordedMessage,
};
use crate::domain_computation::primary_graph::{
    provider::WorthQueryCanonicalInboundCompletion, WorthQueryCommittedDispatchOutboxObservation,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn read_recorded_inbound_owner(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        accepted: &QueryInboundAcceptedMessage,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let correlation = crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity::from_digest(worth_foundational::facade::CanonicalDigestId::new(accepted.claims.correlation_token));
        let owner = self
            .observe_committed_dispatch_outbox_for_correlation(execution, &correlation)
            .map_err(Denial::OwnerReadDenied)?;
        if QueryInboundDispatchSource::from_owner(&owner).map_err(|_| Denial::ForeignOwner)?
            != accepted.source
            || owner.record().operation_slot() != Some(accepted.operation.as_str())
            || owner.record().payload() != accepted.claims.payload
            || owner.record().protocol_identity() != &accepted.claims.protocol_identity
            || owner.record().protocol_version() != accepted.claims.protocol_version
            || owner.record().correlation_family().as_str() != accepted.claims.correlation_family
        {
            return Err(Denial::UnsupportedOutbox);
        }
        Ok(owner)
    }

    pub(super) fn restore_recorded_inbound(
        &self,
        driver: &QueryInboundJournalAdmission<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recorded: &QueryInboundRecordedMessage,
        owner: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<(), Denial> {
        let terminal = self
            .runtime
            .primary_provider
            .lookup_completed_inbound(owner.record().correlation())
            .map_err(|_| Denial::TerminalCleanupUnavailable)?;
        match terminal {
            Some(terminal) => {
                self.restore_recorded_terminal(driver, execution, recorded, &owner, &terminal)
            }
            None => self.restore_recorded_pending(recorded, owner),
        }
    }

    fn restore_recorded_terminal(
        &self,
        driver: &QueryInboundJournalAdmission<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recorded: &QueryInboundRecordedMessage,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        terminal: &WorthQueryCanonicalInboundCompletion,
    ) -> Result<(), Denial> {
        let accepted = &recorded.accepted;
        if !terminal.matches_effect(&accepted.operation, &accepted.claims)
            || !terminal.matches_original_dispatch(
                owner.record(),
                owner.commit_reference(),
                owner.committed_product_publication().composite_commit(),
                owner.committed_product_publication().product_incarnation(),
            )
        {
            return Err(Denial::CorrelationAlreadyOwned);
        }
        let completion = terminal.completion_world_commit().ordinal();
        if recorded
            .acknowledgement
            .is_some_and(|recorded| recorded != completion)
        {
            return Err(Denial::TerminalCleanupUnavailable);
        }
        // Canonical readmitted completion has no fresh delivery to replay.
        driver
            .acknowledge(
                QueryInboundMessageKey::from_claims(&accepted.claims),
                terminal,
                execution,
            )
            .map_err(|_| Denial::RetryBeforeAcceptance)?;
        let contract = owner
            .record()
            .inbound()
            .ok_or(Denial::OriginalDispatchHasNoInboundSupport)?;
        self.runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .retain_recorded_terminal_message(
                &accepted.operation,
                contract,
                &accepted.claims,
                accepted.envelope_bytes,
                owner,
                terminal,
            )
            .map_err(recorded_admission_denial)
    }

    fn restore_recorded_pending(
        &self,
        recorded: &QueryInboundRecordedMessage,
        owner: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<(), Denial> {
        if recorded.acknowledgement.is_some() {
            return Err(Denial::TerminalCleanupUnavailable);
        }
        let accepted = &recorded.accepted;
        let contract = owner
            .record()
            .inbound()
            .ok_or(Denial::OriginalDispatchHasNoInboundSupport)?
            .clone();
        let mut custody = self
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        match custody.accept_retained(
            &accepted.operation,
            &contract,
            accepted.claims.clone(),
            accepted.envelope_bytes,
            owner,
        ) {
            Admission::New(accepted, true) => {
                custody.mark_publication_retryable(&accepted);
                Ok(())
            }
            Admission::New(_, false)
            | Admission::Duplicate(_)
            | Admission::CompactTerminalReplay(_) => Ok(()),
            denied => Err(recorded_admission_denial(denied)),
        }
    }
}
