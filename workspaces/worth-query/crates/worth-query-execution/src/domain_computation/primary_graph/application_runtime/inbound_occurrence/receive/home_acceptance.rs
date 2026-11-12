//! Durability cutover after authentication and exact original-owner admission.
use super::super::home_custody::recorded_admission_denial;
use super::phases::WorthQueryAuthenticatedInboundOccurrence;
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryInboundJournalAdmission, QueryInboundMessageKey,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use WorthQueryInboundAdmissionDenial as Denial;

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(super) fn accept_original_inbound(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        driver: &QueryInboundJournalAdmission<'_>,
        authenticated: &WorthQueryAuthenticatedInboundOccurrence<'_, Schema>,
        owner: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<WorthQueryInboundAdmission, Denial> {
        if let Some(existing) = self.duplicate_inbound_admission(authenticated) {
            return existing;
        }
        admit_active_source(&authenticated.installed)?;
        self.runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .preflight_message(
                &authenticated.operation,
                &authenticated.installed.contract,
                &authenticated.claims,
                authenticated.envelope.len() as u64,
                &owner,
            )
            .map_err(recorded_admission_denial)?;
        self.record_inbound_acceptance(execution, driver, authenticated, &owner)?;
        let mut custody = self
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        authenticated
            .installed
            .cost
            .custody_key_probes
            .fetch_add(1, Ordering::Relaxed);
        let admission = custody.accept(
            &authenticated.operation,
            &authenticated.installed.contract,
            authenticated.claims.clone(),
            authenticated.envelope,
            owner,
        );
        let admitted = super::admit::map_custody(&mut custody, admission)?;
        if let Some(recorded) =
            driver.message(&QueryInboundMessageKey::from_claims(&authenticated.claims))
        {
            custody.durable_admission_cursor = recorded.acceptance_position;
        }
        Ok(admitted)
    }

    pub(super) fn accept_completed_inbound(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        driver: &QueryInboundJournalAdmission<'_>,
        authenticated: &WorthQueryAuthenticatedInboundOccurrence<'_, Schema>,
        posture: WorthQueryInboundReceiptPosture,
    ) -> Result<WorthQueryInboundAdmission, Denial> {
        if let Some(existing) = self.duplicate_inbound_admission(authenticated) {
            return existing;
        }
        if !self.runtime.dispatch_journal.is_durable() {
            return Ok(WorthQueryInboundAdmission::Completed(
                authenticated.claims.clone(),
                posture,
            ));
        }
        admit_active_source(&authenticated.installed)?;
        let correlation = ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new(
            authenticated.claims.correlation_token,
        ));
        let owner = self
            .observe_committed_dispatch_outbox_for_correlation(execution, &correlation)
            .map_err(Denial::OwnerReadDenied)?;
        let terminal = self
            .runtime
            .primary_provider
            .lookup_completed_inbound(&correlation)
            .map_err(|_| Denial::TerminalCleanupUnavailable)?
            .ok_or(Denial::TerminalCleanupUnavailable)?;
        if !terminal.matches_effect(&authenticated.operation, &authenticated.claims)
            || !terminal.matches_original_dispatch(
                owner.record(),
                owner.commit_reference(),
                owner.committed_product_publication().composite_commit(),
                owner.committed_product_publication().product_incarnation(),
            )
        {
            return Err(Denial::CorrelationAlreadyOwned);
        }
        self.runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .preflight_completed_message(
                &authenticated.operation,
                &authenticated.installed.contract,
                &authenticated.claims,
                authenticated.envelope.len() as u64,
                &owner,
            )
            .map_err(recorded_admission_denial)?;
        self.record_completed_inbound_message(execution, driver, authenticated, &owner, &terminal)?;
        Ok(WorthQueryInboundAdmission::Completed(
            authenticated.claims.clone(),
            posture,
        ))
    }

    fn duplicate_inbound_admission(
        &self,
        authenticated: &WorthQueryAuthenticatedInboundOccurrence<'_, Schema>,
    ) -> Option<Result<WorthQueryInboundAdmission, Denial>> {
        let existing = self
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .duplicate(
                &authenticated.operation,
                &authenticated.claims,
                authenticated.envelope,
            )?;
        let mut custody = self
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        Some(super::admit::map_custody(&mut custody, existing))
    }

    fn record_inbound_acceptance(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        driver: &QueryInboundJournalAdmission<'_>,
        authenticated: &WorthQueryAuthenticatedInboundOccurrence<'_, Schema>,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<(), Denial> {
        if !self.runtime.dispatch_journal.is_durable() {
            return Ok(());
        }
        driver
            .record_acceptance(authenticated, owner, execution)
            .map_err(|_| Denial::RetryBeforeAcceptance)
    }

    fn record_completed_inbound_message(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        driver: &QueryInboundJournalAdmission<'_>,
        authenticated: &WorthQueryAuthenticatedInboundOccurrence<'_, Schema>,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
        terminal: &crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion,
    ) -> Result<(), Denial> {
        self.record_inbound_acceptance(execution, driver, authenticated, owner)?;
        let key = QueryInboundMessageKey::from_claims(&authenticated.claims);
        driver
            .acknowledge(key.clone(), terminal, execution)
            .map_err(|_| Denial::RetryBeforeAcceptance)?;
        let mut custody = self
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        custody
            .retain_recorded_terminal_message(
                &authenticated.operation,
                &authenticated.installed.contract,
                &authenticated.claims,
                authenticated.envelope.len() as u64,
                owner,
                terminal,
            )
            .map_err(recorded_admission_denial)?;
        custody.durable_admission_cursor = driver
            .message(&key)
            .expect("committed acceptance")
            .acceptance_position;
        Ok(())
    }
}

fn admit_active_source(
    installed: &super::super::WorthQueryInstalledInboundVerifier,
) -> Result<(), Denial> {
    match installed.source_posture() {
        WorthQueryInboundSourcePosture::Active => Ok(()),
        WorthQueryInboundSourcePosture::Retired => Err(Denial::SourceRetired),
        WorthQueryInboundSourcePosture::Revoked => Err(Denial::SourceRevoked),
        WorthQueryInboundSourcePosture::ControlPending => Err(Denial::RetryBeforeAcceptance),
    }
}
