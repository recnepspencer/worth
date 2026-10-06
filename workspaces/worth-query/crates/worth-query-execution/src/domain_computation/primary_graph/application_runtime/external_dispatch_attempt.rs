//! Runtime-owned admission for one physical post-commit dispatch attempt.

use std::sync::Arc;

use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;

/// Why this application runtime could not admit a physical dispatch attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryExternalDispatchAdmissionDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    PublicationCommitMismatch,
    MissingInboundOperationSlot,
    AlreadyCompleted,
    CompletedTransportRetained,
    TerminalIndexUnavailable,
    AttemptIdentityExhausted,
}

/// The one terminal effect owner refused a new physical attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum WorthQueryTerminalEffectRefusal {
    AlreadyCompleted,
    TerminalIndexUnavailable,
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

/// Opaque, move-only ordinal minted only while the runtime admits an attempt.
pub(in crate::domain_computation) struct WorthQueryExternalDispatchAttemptOrdinal(u64);

impl WorthQueryExternalDispatchAttemptOrdinal {
    fn mint(value: u64) -> Self {
        Self(value)
    }

    pub(in crate::domain_computation) fn into_value(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(in crate::domain_computation) const fn value_for_test(&self) -> u64 {
        self.0
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub(in crate::domain_computation::primary_graph) fn admit_external_dispatch_attempt(
        &self,
        committed: WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<WorthQueryAdmittedExternalDispatchAttempt, WorthQueryExternalDispatchAdmissionDenial>
    {
        self.primary_provider.observe_external_dispatch_admission();
        let relational_runtime = self
            .product_runtime
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
            != self.product_runtime.owner.owner_identity()
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
            self.next_external_dispatch_attempt
                .fetch_update(
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Acquire,
                    |current| current.checked_add(1),
                )
                .map_err(|_| WorthQueryExternalDispatchAdmissionDenial::AttemptIdentityExhausted)?,
        );
        Ok(WorthQueryAdmittedExternalDispatchAttempt::seal(
            committed,
            self.runtime.authority_identity(),
            ordinal,
            Arc::clone(&self.authorization_clock),
        ))
    }

    /// Ask the one terminal effect owner before any physical attempt.
    ///
    /// Once World has performed the terminal transition, no new physical
    /// attempt may be admitted for this original effect. An attempt admitted
    /// before that transition may already be on the wire; the installed rail
    /// idempotency contract resolves that race. The committed outbox row may
    /// already be released, so this reads only the terminal owner.
    pub(in crate::domain_computation::primary_graph) fn refuse_completed_external_effect(
        &self,
        record: &WorthQueryDispatchOutboxRecord,
    ) -> Result<(), WorthQueryTerminalEffectRefusal> {
        if self
            .inbound_custody
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
            .primary_provider
            .lookup_completed_inbound(record.correlation())
        {
            Ok(Some(_)) => Err(WorthQueryTerminalEffectRefusal::AlreadyCompleted),
            Ok(None) => Ok(()),
            Err(_) => Err(WorthQueryTerminalEffectRefusal::TerminalIndexUnavailable),
        }
    }
}
