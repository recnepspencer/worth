//! One source-admitted, journal-owned physical send and its completion handoff.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_external_effect;
use crate::domain_computation::primary_graph::{
    InstalledTransportCompletion, InstalledTransportResumeOutcome,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn perform_committed_external_dispatch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        transport: &dyn WorthQueryExternalEffectTransport,
        committed: crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        mut observe: impl FnMut(OriginalDispatchObservation<'_>),
    ) -> Result<WorthQueryExternalEffectDispatch, WorthQueryExternalDispatchPreparationDenial> {
        if committed.record().inbound().is_some()
            && self.has_retained_installed_transport_completion(committed.record().correlation())
        {
            let resumed = self.resume_installed_transport_completion(
                execution,
                phase,
                committed.record().correlation(),
                request,
            );
            if matches!(resumed, InstalledTransportResumeOutcome::Performed) {
                let _ = self.progress_retained_inbound_occurrence(
                    execution,
                    phase,
                    *committed.record().correlation().bytes(),
                    request,
                );
            }
            return Err(match resumed {
                InstalledTransportResumeOutcome::Performed => {
                    WorthQueryExternalDispatchPreparationDenial::AlreadyCompleted
                }
                InstalledTransportResumeOutcome::Pending(reason) => {
                    reason.dispatch_preparation_denial()
                }
            });
        }
        let completion_owner = committed.clone();
        let prepared = self
            .admit_external_dispatch_attempt(committed)
            .map_err(denial::from_attempt_admission)?;
        let in_flight = self
            .runtime
            .primary_provider
            .begin_external_dispatch_in_flight(&completion_owner)
            .map_err(denial::from_in_flight)?;
        // Durable before contact: a refused record returns before the transport.
        let contact = self.begin_durable_dispatch_contact(execution, completion_owner.record())?;
        let admitted = if contact.is_recorded() {
            prepared
                .admit_recorded(&contact)
                .map_err(denial::from_attempt_admission)?
        } else {
            prepared
                .admit_memory()
                .map_err(denial::from_attempt_admission)?
        };
        let dispatch = match dispatch_external_effect(transport, admitted, || {
            observe(OriginalDispatchObservation::Starting);
            contact.mark_contact_started()
        }) {
            Ok(dispatch) => dispatch,
            Err(denial) => {
                use crate::domain_computation::application_aftermath::dispatch_journal::{
                    ExternalEffectNoEffectCause, QueryDispatchRecordedObservation,
                };
                use crate::domain_computation::application_aftermath::WorthQueryAftermathDerivationFailure as Failure;
                // Every derivation failure precedes the transport call.
                contact.observe(
                    execution,
                    QueryDispatchRecordedObservation::ProvenNoEffect(
                        ExternalEffectNoEffectCause::ContactNotStarted,
                    ),
                );
                return Err(match denial {
                    Failure::RuntimeTimeUnavailable => {
                        WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied
                    }
                    Failure::RetentionCapacityExhausted
                    | Failure::RetentionIdentityExhausted
                    | Failure::SnapshotIdentityExhausted
                    | Failure::BasisRejected
                    | Failure::DigestRejected
                    | Failure::EmptyCorrelationBasis
                    | Failure::MissingExternalPayload => {
                        WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied
                    }
                });
            }
        };
        // Durable before live: the record precedes every observer of the outcome.
        contact.observe(execution, durable_attempt::recorded_observation(&dispatch));
        observe(OriginalDispatchObservation::Observed(&dispatch));
        if completion_owner.record().inbound().is_some() && dispatch.is_external_completion() {
            let evidence =
                InstalledTransportCompletion::from_observed_dispatch(completion_owner, &dispatch)
                    .map_err(|_| {
                    WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied
                })?;
            let in_flight = in_flight.expect("inbound dispatch reserved an in-flight slot");
            // The original outbox reserved finite provenance before transport.
            // Keep its actual completion observation even if World needs retry.
            let _ = self.record_installed_transport_completion(evidence, in_flight);
            if matches!(
                self.resume_installed_transport_completion(
                    execution,
                    phase,
                    dispatch.correlation(),
                    request
                ),
                InstalledTransportResumeOutcome::Performed,
            ) {
                // A callback may have entered signed custody while the
                // physical send was still in flight. Transport terminal truth
                // now owns the effect; release that redundant accepted slot.
                let _ = self.progress_retained_inbound_occurrence(
                    execution,
                    phase,
                    *dispatch.correlation().bytes(),
                    request,
                );
            }
        }
        Ok(dispatch)
    }
}
