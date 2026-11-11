//! The runtime's one dispatch-custody journal owner: cold admission before
//! service, an attempt recorded before contact, one observation after it.
use super::*;
use std::sync::{Arc, Mutex, OnceLock};
use worth_execution::ExecutionRequest;
mod inbound_admission;
mod settlement;
mod source_controls;
mod transport;
pub(in crate::domain_computation) use inbound_admission::QueryInboundJournalAdmission;
use settlement::DispatchJournalDriver;
pub(in crate::domain_computation) use source_controls::QueryInboundSourceControlObservation;

/// A memory-only runtime keeps this owner unattached: its dispatch custody is
/// process-local and every durable record request answers `JournalUnavailable`.
/// Every durable fact enters through the medium's Home-issued proofs, so
/// holding a journal grants no authority over any Home.
pub struct WorthQueryDispatchJournal {
    state: Mutex<Option<DispatchJournalState>>,
    medium: OnceLock<Arc<dyn QueryDispatchMedium>>,
    publication_driver: Mutex<()>,
    process_local_source_controls: Mutex<BTreeMap<CanonicalDigestId, InboundSourceControl>>,
}

struct DispatchJournalState {
    limits: super::read_limits::QueryDispatchReadLimits,
    ledger: QueryDispatchLedger,
    generations: QueryDispatchPublicationGenerations,
    /// The one original frame whose physical fate is not yet folded.
    pending: Option<QueryDispatchFrame>,
    /// Contacts already observed whose observation is not yet durable.
    unrecorded: BTreeMap<DispatchAttemptIdentity, QueryDispatchRecordedObservation>,
}

impl std::fmt::Debug for WorthQueryDispatchJournal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryDispatchJournal")
            .field("durable", &self.is_durable())
            .finish_non_exhaustive()
    }
}

impl WorthQueryDispatchJournal {
    /// An unattached owner; `readmit` attaches the one Home journal it serves.
    pub(in crate::domain_computation) fn process_local() -> Self {
        Self {
            state: Mutex::new(None),
            medium: OnceLock::new(),
            publication_driver: Mutex::new(()),
            process_local_source_controls: Mutex::new(BTreeMap::new()),
        }
    }

    /// True once this runtime's Home journal was readmitted before service.
    pub fn is_durable(&self) -> bool {
        self.medium.get().is_some()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, Option<DispatchJournalState>> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn installed_medium(
        &self,
    ) -> Result<&Arc<dyn QueryDispatchMedium>, QueryDispatchJournalDenial> {
        self.medium
            .get()
            .ok_or(QueryDispatchJournalDenial::JournalUnavailable)
    }

    /// Cold admission of the complete recorded history, before Home service.
    /// It folds DATA only: no attempt is contacted, replayed or re-recorded.
    pub(in crate::domain_computation) fn readmit(
        &self,
        medium: Arc<dyn QueryDispatchMedium>,
        request: &ExecutionRequest<'_, '_>,
        catalog: &worth_query_installation::facade::InstalledOperationCatalog,
    ) -> Result<worth_proof::AdmittedHomeQueryDispatch, QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial as Denial;
        let driver = DispatchJournalDriver::lock(self, request)?;
        let medium = if let Some(installed) = self.medium.get() {
            installed
                .authority()
                .admit(medium.authority())
                .map_err(incarnation_denial)?;
            Arc::clone(installed)
        } else {
            let limits = super::read_limits::QueryDispatchReadLimits::from_catalog(catalog);
            let mut ledger = QueryDispatchLedger::empty(medium.home());
            while let Some(bytes) = medium.read_frame(
                request,
                ledger
                    .position()
                    .checked_add(1)
                    .ok_or(Denial::IdentityExhausted)?,
                limits.maximum_frame_bytes(),
            )? {
                request.consult().map_err(Denial::Execution)?;
                let frame = super::codec::decode(&bytes, limits.maximum_frame_bytes())?;
                limits.validate(&frame)?;
                ledger.readmit(frame)?;
            }
            if let Some(unobserved) = ledger.first_unobserved() {
                return Err(unobserved);
            }
            let mut state = self.state();
            self.medium
                .set(Arc::clone(&medium))
                .map_err(|_| Denial::OwnerUnavailable)?;
            *state = Some(DispatchJournalState {
                limits,
                generations: QueryDispatchPublicationGenerations::from_journal(&ledger),
                ledger,
                pending: None,
                unrecorded: BTreeMap::new(),
            });
            medium
        };
        driver.settle(medium.as_ref(), request)?;
        let state = self.state();
        let ledger = &state.as_ref().expect("admitted dispatch journal").ledger;
        Ok(medium.authority().admit_query_dispatch_prefix(
            ledger.position(),
            ledger.digest(),
            ledger.payload_digest(),
        ))
    }

    /// Record the attempt durably. Only an `Ok` permits its external contact.
    ///
    /// Whatever an earlier request left behind is settled first, on this
    /// request. If this request stops before its own frame's fate is known,
    /// the frame stays retained: the next dispatch or graceful close settles
    /// it, and a frame that still commits is observed as `ContactNotStarted`.
    pub(in crate::domain_computation) fn record_attempt(
        &self,
        correlation: CanonicalDigestId,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<DispatchAttemptIdentity, QueryDispatchJournalDenial> {
        let medium = Arc::clone(self.installed_medium()?);
        let driver = DispatchJournalDriver::lock(self, request)?;
        driver.settle(medium.as_ref(), request)?;
        match driver.publish(
            medium.as_ref(),
            QueryDispatchRecordIntent::Attempt { correlation },
            request,
        )? {
            QueryDispatchRecord::Attempt(attempt) => Ok(attempt.identity),
            QueryDispatchRecord::Observation { .. }
            | QueryDispatchRecord::SourceControl { .. }
            | QueryDispatchRecord::Inbound(_) => {
                unreachable!("an attempt intent encodes an attempt record")
            }
        }
    }

    /// Retain the attempt's one observation and make it durable on the
    /// invoking request. A refusal keeps the observation for the next
    /// settlement; it is never dropped, replaced or re-derived. Until it is
    /// durable the Home is not quiescent: graceful close settles it first,
    /// and a process that dies before then reopens as an unclean dispatch.
    pub(in crate::domain_computation) fn record_observation(
        &self,
        attempt: DispatchAttemptIdentity,
        observation: QueryDispatchRecordedObservation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        let medium = Arc::clone(self.installed_medium()?);
        self.retain(attempt, observation)?;
        DispatchJournalDriver::lock(self, request)?.settle(medium.as_ref(), request)
    }

    /// Retain an observation without any physical work, for a contact that
    /// is unwinding. The first retained observation of an attempt stands.
    pub(in crate::domain_computation) fn retain_observation(
        &self,
        attempt: DispatchAttemptIdentity,
        observation: QueryDispatchRecordedObservation,
    ) {
        let _first_retained_observation_stands = self.retain(attempt, observation);
    }

    fn retain(
        &self,
        attempt: DispatchAttemptIdentity,
        observation: QueryDispatchRecordedObservation,
    ) -> Result<(), QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial as Denial;
        let mut guard = self.state();
        let state = guard.as_mut().ok_or(Denial::JournalUnavailable)?;
        if !state.ledger.awaits_observation(&attempt) {
            return Err(Denial::UnknownAttempt);
        }
        match state.unrecorded.get(&attempt) {
            Some(retained) if *retained != observation => Err(Denial::AttemptAlreadyObserved),
            Some(_) => Ok(()),
            None => {
                state.unrecorded.insert(attempt, observation);
                Ok(())
            }
        }
    }

    /// Finish every retained publication and observation. Graceful close
    /// requires `Ok`: an attempt still inside its contact refuses.
    pub(in crate::domain_computation) fn settle_for_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<QueryDispatchCloseCustody, QueryDispatchJournalDenial> {
        let Some(medium) = self.medium.get().map(Arc::clone) else {
            return Ok(QueryDispatchCloseCustody::ProcessLocal);
        };
        let driver = DispatchJournalDriver::lock(self, request)?;
        driver.settle(medium.as_ref(), request)?;
        let guard = self.state();
        let state = guard.as_ref().expect("admitted dispatch journal");
        match state.ledger.first_unobserved() {
            Some(unobserved) => Err(unobserved),
            None => Ok(QueryDispatchCloseCustody::Durable(
                medium.authority().admit_query_dispatch_prefix(
                    state.ledger.position(),
                    state.ledger.digest(),
                    state.ledger.payload_digest(),
                ),
            )),
        }
    }

    /// The latest durable custody of one effect. `None` for a memory-only
    /// runtime and for an effect no attempt has been recorded for.
    pub(in crate::domain_computation) fn custody(
        &self,
        correlation: &CanonicalDigestId,
    ) -> Option<QueryDispatchCustody> {
        self.state()
            .as_ref()
            .and_then(|state| state.ledger.custody(correlation))
    }
}

fn incarnation_denial(cause: worth_proof::HomeIncarnationDenial) -> QueryDispatchJournalDenial {
    match cause {
        worth_proof::HomeIncarnationDenial::ForeignHome => QueryDispatchJournalDenial::ForeignHome,
        worth_proof::HomeIncarnationDenial::StaleIncarnation => {
            QueryDispatchJournalDenial::StaleIncarnation
        }
    }
}
