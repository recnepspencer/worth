//! The Home journal records a physical attempt before contact and its one observation after.
use worth_foundational::facade::DispatchAttemptIdentity;
use worth_query_installation::facade::ApplicationSchema;

use super::denial::{
    WorthQueryExternalDispatchAttemptDenial, WorthQueryExternalDispatchPreparationDenial,
};
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryDispatchJournalDenial, QueryDispatchProvenNoEffect, QueryDispatchRecordedObservation,
    QueryDispatchUnresolvedCause, WorthQueryDispatchJournal,
};
use crate::domain_computation::application_aftermath::{
    ExternalRailTransportFault, WorthQueryDispatchOutboxRecord,
    WorthQueryExternalDispatchPostureKind, WorthQueryExternalEffectDispatch,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// Why the Home's dispatch journal did not record a physical attempt. In every
/// case the external contact never began.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDurableDispatchRecordDenial {
    /// An earlier attempt for this effect still awaits its recorded observation.
    AttemptAwaitingObservation,
    /// The invoking request stopped before the record's fate was known.
    RequestStopped,
    /// The record's physical fate is unknown until the Home reopens.
    PublicationUnresolved,
    /// The record was proven unwritten; a later attempt records afresh.
    PublicationNoEffect,
    /// The Home journal refused the record or is no longer available.
    JournalRefused,
}

/// One permitted external contact. Its attempt is already durable, or the
/// runtime is memory-only and its custody is process-local. Exactly one
/// observation follows: the one the contact reports, or, if the contact
/// unwinds first, the truthful unknown outcome.
pub(super) struct DurableDispatchContact<'journal> {
    journal: &'journal WorthQueryDispatchJournal,
    attempt: Option<DispatchAttemptIdentity>,
}

impl DurableDispatchContact<'_> {
    /// The journal owner makes the observation durable on the invoking
    /// request and retains it across any refusal, including that request
    /// having stopped; graceful close refuses until it is recorded.
    pub(super) fn observe(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: QueryDispatchRecordedObservation,
    ) {
        if let Some(attempt) = self.attempt.take() {
            let _retained_until_settlement =
                self.journal
                    .record_observation(attempt, observation, execution);
        }
    }
}

impl Drop for DurableDispatchContact<'_> {
    fn drop(&mut self) {
        if let Some(attempt) = self.attempt.take() {
            // No physical work while unwinding: the next settlement records it.
            self.journal.retain_observation(
                attempt,
                QueryDispatchRecordedObservation::Unresolved(
                    QueryDispatchUnresolvedCause::UnknownProviderOutcome,
                ),
            );
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Only a memory-only owner dispatches without a durable record. A durable
    /// owner whose journal is not attached refuses before any contact.
    pub(super) fn begin_durable_dispatch_contact(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &WorthQueryDispatchOutboxRecord,
    ) -> Result<DurableDispatchContact<'_>, WorthQueryExternalDispatchPreparationDenial> {
        let journal = &self.runtime.dispatch_journal;
        let attempt = if journal.is_durable() {
            Some(
                journal
                    .record_attempt(*record.correlation().digest(), execution)
                    .map_err(attempt_record_denial)?,
            )
        } else if self
            .runtime
            .product_runtime
            .owner
            .owner_identity()
            .durable()
            .is_some()
        {
            return Err(attempt_record_denial(
                QueryDispatchJournalDenial::JournalUnavailable,
            ));
        } else {
            None
        };
        Ok(DurableDispatchContact { journal, attempt })
    }

    /// The latest custody this Home's dispatch journal durably records for
    /// the effect with this correlation token (the token its outbox row and
    /// every transport request carry). `None` for a memory-only runtime and
    /// for an effect no attempt has been recorded for. Recorded DATA: it
    /// grants no dispatch authority.
    pub fn durable_dispatch_custody(
        &self,
        correlation_token: &[u8; 32],
    ) -> Option<
        crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchCustody,
    > {
        self.runtime
            .dispatch_journal
            .custody(&worth_foundational::facade::CanonicalDigestId::new(
                *correlation_token,
            ))
    }
}

/// The durable observation of one classified transport contact.
pub(super) fn recorded_observation(
    dispatch: &WorthQueryExternalEffectDispatch,
) -> QueryDispatchRecordedObservation {
    use QueryDispatchRecordedObservation as Observation;
    use QueryDispatchUnresolvedCause as Cause;
    match dispatch.posture().kind() {
        WorthQueryExternalDispatchPostureKind::Completed => Observation::Completed,
        WorthQueryExternalDispatchPostureKind::Acknowledged => Observation::Acknowledged,
        WorthQueryExternalDispatchPostureKind::Unresolved => match dispatch
            .fault()
            .expect("an unresolved posture carries its classified fault")
        {
            ExternalRailTransportFault::PayloadRejected => {
                Observation::ProvenNoEffect(QueryDispatchProvenNoEffect::PayloadRejected)
            }
            ExternalRailTransportFault::Timeout => Observation::Unresolved(Cause::Timeout),
            ExternalRailTransportFault::Disconnect => Observation::Unresolved(Cause::Disconnect),
            ExternalRailTransportFault::LostResponse => {
                Observation::Unresolved(Cause::LostResponse)
            }
            ExternalRailTransportFault::DuplicatedAcknowledgement => {
                Observation::Unresolved(Cause::DuplicatedAcknowledgement)
            }
            ExternalRailTransportFault::UnsupportedProtocolVersion(_) => {
                Observation::Unresolved(Cause::UnsupportedProtocolVersion)
            }
            ExternalRailTransportFault::UnknownProviderOutcome => {
                Observation::Unresolved(Cause::UnknownProviderOutcome)
            }
            ExternalRailTransportFault::ObservationDerivationDenied => {
                Observation::Unresolved(Cause::ObservationDerivationDenied)
            }
        },
    }
}

fn attempt_record_denial(
    cause: QueryDispatchJournalDenial,
) -> WorthQueryExternalDispatchPreparationDenial {
    use QueryDispatchJournalDenial as Journal;
    use WorthQueryDurableDispatchRecordDenial as Record;
    let record = match cause {
        Journal::EffectAlreadyCompleted => {
            return WorthQueryExternalDispatchPreparationDenial::AlreadyCompleted
        }
        Journal::AttemptAwaitingObservation => Record::AttemptAwaitingObservation,
        Journal::Execution(_) => Record::RequestStopped,
        Journal::PublicationUnresolved => Record::PublicationUnresolved,
        Journal::PublicationNoEffect => Record::PublicationNoEffect,
        Journal::UnsupportedVersion(_)
        | Journal::MalformedFrame
        | Journal::Checksum
        | Journal::ForeignHome
        | Journal::PrefixMismatch
        | Journal::IdentityExhausted
        | Journal::AttemptIdentityMismatch
        | Journal::UnknownAttempt
        | Journal::AttemptAlreadyObserved
        | Journal::UnobservedAttempt { .. }
        | Journal::JournalUnavailable
        | Journal::OwnerUnavailable
        | Journal::StaleIncarnation
        | Journal::PublicationPending
        | Journal::PublicationGenerationMismatch
        | Journal::Storage(_) => Record::JournalRefused,
    };
    WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied(
        WorthQueryExternalDispatchAttemptDenial::DurableAttemptRecordDenied(record),
    )
}
