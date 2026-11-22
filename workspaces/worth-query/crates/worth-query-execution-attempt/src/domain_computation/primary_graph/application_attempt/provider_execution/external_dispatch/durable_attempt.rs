//! The Home journal records a physical attempt before contact and its one observation after.
use worth_foundational::facade::DispatchAttemptIdentity;

use super::denial::{
    WorthQueryExternalDispatchAttemptDenial, WorthQueryExternalDispatchPreparationDenial,
};
use crate::domain_computation::application_aftermath::dispatch_journal::{
    QueryDispatchJournalDenial, QueryDispatchRecordedObservation, QueryDispatchUnresolvedCause,
    WorthQueryDispatchJournal,
};
use crate::domain_computation::application_aftermath::{
    ExternalRailTransportFault, WorthQueryExternalDispatchPostureKind,
    WorthQueryExternalEffectDispatch,
};

pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::provider_execution::external_dispatch::durable_attempt::WorthQueryDurableDispatchRecordDenial;

/// One permitted external contact. Its attempt is already durable, or the
/// runtime is memory-only and its custody is process-local. Exactly one
/// observation follows: the one the contact reports, or, if the contact
/// unwinds first, the truthful unknown outcome.
pub struct DurableDispatchContact<'journal> {
    pub journal: &'journal WorthQueryDispatchJournal,
    pub attempt: Option<DispatchAttemptIdentity>,
    pub correlation: worth_foundational::facade::CanonicalDigestId,
    contact_started: std::sync::atomic::AtomicBool,
}

impl<'journal> DurableDispatchContact<'journal> {
    pub fn new(
        journal: &'journal WorthQueryDispatchJournal,
        attempt: Option<DispatchAttemptIdentity>,
        correlation: worth_foundational::facade::CanonicalDigestId,
    ) -> Self {
        Self {
            journal,
            attempt,
            correlation,
            contact_started: std::sync::atomic::AtomicBool::new(false),
        }
    }
    pub fn is_recorded(&self) -> bool {
        self.attempt.is_some()
    }
    pub fn recorded_attempt(&self) -> DispatchAttemptIdentity {
        self.attempt
            .expect("recorded contact owns its actual attempt")
    }
    pub fn correlation(&self) -> &worth_foundational::facade::CanonicalDigestId {
        &self.correlation
    }
    pub fn mark_contact_started(&self) {
        self.contact_started
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// The journal owner makes the observation durable on the invoking
    /// request and retains it across any refusal, including that request
    /// having stopped; graceful close refuses until it is recorded.
    pub fn observe(
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
                if self
                    .contact_started
                    .load(std::sync::atomic::Ordering::Acquire)
                {
                    QueryDispatchRecordedObservation::Unresolved(
                        QueryDispatchUnresolvedCause::UnknownProviderOutcome,
                    )
                } else {
                    QueryDispatchRecordedObservation::ProvenNoEffect(
                        worth_foundational::facade::ExternalEffectNoEffectCause::ContactNotStarted,
                    )
                },
            );
        }
    }
}

pub fn recorded_observation(
    dispatch: &WorthQueryExternalEffectDispatch,
) -> QueryDispatchRecordedObservation {
    use QueryDispatchRecordedObservation as Observation;
    use QueryDispatchUnresolvedCause as Cause;
    match dispatch.posture().kind() {
        WorthQueryExternalDispatchPostureKind::Completed => Observation::Completed,
        WorthQueryExternalDispatchPostureKind::Acknowledged => Observation::Acknowledged,
        WorthQueryExternalDispatchPostureKind::ProvenNoEffect => Observation::ProvenNoEffect(
            dispatch
                .posture()
                .proven_no_effect()
                .expect("a proven-no-effect posture retains its exact proof")
                .cause(),
        ),
        WorthQueryExternalDispatchPostureKind::Unresolved => match dispatch
            .fault()
            .expect("an unresolved posture carries its classified fault")
        {
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

pub fn attempt_record_denial(
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
