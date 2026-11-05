//! Runtime-owned admission for one physical post-commit dispatch attempt.

use crate::domain_computation::primary_graph::application_inbound::CustodyScopedOps;
use std::sync::Arc;

use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
use crate::domain_computation::application_aftermath::external_effect::WorthQueryTerminalEffectRefusal;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

/// Why this application runtime could not admit a physical dispatch attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchAdmissionDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    PublicationCommitMismatch,
    MissingInboundOperationSlot,
    AlreadyCompleted,
    CompletedTransportRetained,
    TerminalIndexUnavailable,
    AttemptIdentityExhausted,
}

impl From<WorthQueryTerminalEffectRefusal> for WorthQueryExternalDispatchAdmissionDenial {
    fn from(refusal: WorthQueryTerminalEffectRefusal) -> Self {
        match refusal {
            WorthQueryTerminalEffectRefusal::AlreadyCompleted => Self::AlreadyCompleted,
            WorthQueryTerminalEffectRefusal::TerminalIndexUnavailable => {
                Self::TerminalIndexUnavailable
            }
        }
    }
}



pub(in crate::domain_computation::primary_graph) trait ExternalDispatchAttemptScopedOps<
    Schema: worth_query_installation::facade::ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    fn admit_external_dispatch_attempt(
        &self,
        committed: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<WorthQueryAdmittedExternalDispatchAttempt, WorthQueryExternalDispatchAdmissionDenial>
    {
        self.primary_provider()
            .observe_external_dispatch_admission();
        let relational_runtime = self
            .product_runtime()
            .source
            .authoritative_source_profile()
            .runtime_instance_id();
        if committed.relational_runtime_instance_id() != relational_runtime {
            return Err(WorthQueryExternalDispatchAdmissionDenial::ForeignRelationalRuntime);
        }
        if committed
            .committed_product_publication()
            .product_branch()
            .owner_identity()
            != self.product_runtime().owner.owner_identity()
        {
            return Err(WorthQueryExternalDispatchAdmissionDenial::ForeignProductWorld);
        }
        if committed.commit_reference()
            != committed
                .committed_product_publication()
                .relational_commit()
        {
            return Err(WorthQueryExternalDispatchAdmissionDenial::PublicationCommitMismatch);
        }
        // Installed inbound completion needs the original operation's exact
        // co-committed slot. A restored legacy row without it cannot enter the
        // physical rail and leave a completion that owner maintenance cannot find.
        if committed.record().inbound().is_some() && committed.record().operation_slot().is_none() {
            return Err(WorthQueryExternalDispatchAdmissionDenial::MissingInboundOperationSlot);
        }
        if committed.record().inbound().is_some()
            && self.has_retained_installed_transport_completion(committed.record().correlation())
        {
            return Err(WorthQueryExternalDispatchAdmissionDenial::CompletedTransportRetained);
        }
        self.refuse_completed_external_effect(committed.record())?;
        let ordinal = WorthQueryExternalDispatchAttemptOrdinal::mint(
            {
                let update_counter = self.next_external_dispatch_attempt();
                let mut current = update_counter.load(std::sync::atomic::Ordering::Acquire);
                loop {
                    let Some(updated_value) = current.checked_add(1) else {
                        break Err(current);
                    };
                    match update_counter.compare_exchange_weak(
                        current,
                        updated_value,
                        std::sync::atomic::Ordering::AcqRel,
                        std::sync::atomic::Ordering::Acquire,
                    ) {
                        Ok(previous) => break Ok(previous),
                        Err(reloaded_value) => current = reloaded_value,
                    }
                }
            }
            .map_err(|_| WorthQueryExternalDispatchAdmissionDenial::AttemptIdentityExhausted)?,
        );
        Ok(WorthQueryAdmittedExternalDispatchAttempt::seal(
            committed,
            self.authority_identity(),
            ordinal,
            Arc::clone(self.authorization_clock()),
        ))
    }

    fn refuse_completed_external_effect(
        &self,
        record: &WorthQueryDispatchOutboxRecord,
    ) -> Result<(), WorthQueryTerminalEffectRefusal> {
        if self
            .inbound()
            .custody()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .terminal_by_correlation(record.correlation())
            .is_some()
        {
            return Err(WorthQueryTerminalEffectRefusal::AlreadyCompleted);
        }
        if record.inbound().is_none() {
            return Ok(());
        }
        match self
            .primary_provider()
            .lookup_completed_inbound(record.correlation())
        {
            Ok(Some(_)) => Err(WorthQueryTerminalEffectRefusal::AlreadyCompleted),
            Ok(None) => Ok(()),
            Err(_) => Err(WorthQueryTerminalEffectRefusal::TerminalIndexUnavailable),
        }
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ExternalDispatchAttemptScopedOps<Schema> for HostRuntime {}

pub(crate) mod court_doors_for_test;
pub use court_doors_for_test::ExternalDispatchAttemptDoorsOps;
use crate::domain_computation::vocabulary::primary_graph::external_dispatch_attempt::WorthQueryExternalDispatchAttemptOrdinal;
