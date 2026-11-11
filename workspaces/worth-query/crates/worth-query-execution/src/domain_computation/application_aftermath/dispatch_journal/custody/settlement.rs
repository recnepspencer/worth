//! One serialized original publication at a time, with no state lock across IO.
use super::*;

/// The one original publication the driver found or started.
enum PendingFate {
    Absent,
    Committed(QueryDispatchRecord),
    ProvenNoEffect,
}

pub(super) struct DispatchJournalDriver<'a> {
    journal: &'a WorthQueryDispatchJournal,
    _driver: std::sync::MutexGuard<'a, ()>,
}

impl<'a> DispatchJournalDriver<'a> {
    /// A waiter keeps consulting its own request, so none outlives its deadline.
    pub(super) fn lock(
        journal: &'a WorthQueryDispatchJournal,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, QueryDispatchJournalDenial> {
        loop {
            match journal.publication_driver.try_lock() {
                Ok(guard) => {
                    return Ok(Self {
                        journal,
                        _driver: guard,
                    })
                }
                Err(std::sync::TryLockError::Poisoned(cause)) => {
                    return Ok(Self {
                        journal,
                        _driver: cause.into_inner(),
                    })
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    request
                        .consult()
                        .map_err(QueryDispatchJournalDenial::Execution)?;
                    std::thread::yield_now();
                }
            }
        }
    }

    /// Finish what earlier requests left behind, then every retained observation.
    ///
    /// A frame found here lost its requesting caller: that caller received a
    /// refusal and never began the contact. If the frame is an attempt that
    /// still commits, its truthful observation is `ContactNotStarted`.
    pub(super) fn settle(
        &self,
        medium: &dyn QueryDispatchMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        match self.finish_pending(medium, request)? {
            PendingFate::Committed(QueryDispatchRecord::Attempt(attempt)) => {
                self.journal
                    .state()
                    .as_mut()
                    .expect("admitted dispatch journal")
                    .unrecorded
                    .insert(
                        attempt.identity,
                        QueryDispatchRecordedObservation::ProvenNoEffect(
                            ExternalEffectNoEffectCause::ContactNotStarted,
                        ),
                    );
            }
            PendingFate::Committed(
                QueryDispatchRecord::Observation { .. } | QueryDispatchRecord::SourceControl { .. },
            )
            | PendingFate::ProvenNoEffect
            | PendingFate::Absent => {}
        }
        self.record_retained_observations(medium, request)
    }

    fn record_retained_observations(
        &self,
        medium: &dyn QueryDispatchMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        loop {
            let next = {
                let mut guard = self.journal.state();
                let state = guard.as_mut().expect("admitted dispatch journal");
                let Some((attempt, observation)) = state
                    .unrecorded
                    .iter()
                    .next()
                    .map(|(attempt, observation)| (*attempt, *observation))
                else {
                    return Ok(());
                };
                if state.ledger.awaits_observation(&attempt) {
                    Some((attempt, observation))
                } else {
                    state.unrecorded.remove(&attempt);
                    None
                }
            };
            if let Some((attempt, observation)) = next {
                self.publish(
                    medium,
                    QueryDispatchRecordIntent::Observation {
                        attempt,
                        observation,
                    },
                    request,
                )?;
            }
        }
    }

    /// Prepare one new original frame and drive it to its actual fate.
    pub(super) fn publish(
        &self,
        medium: &dyn QueryDispatchMedium,
        intent: QueryDispatchRecordIntent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<QueryDispatchRecord, QueryDispatchJournalDenial> {
        {
            let mut guard = self.journal.state();
            let state = guard.as_mut().expect("admitted dispatch journal");
            let frame = state.ledger.prepare(intent, &mut state.generations)?;
            state.pending = Some(frame);
        }
        match self.finish_pending(medium, request)? {
            PendingFate::Committed(record) => Ok(record),
            PendingFate::ProvenNoEffect => Err(QueryDispatchJournalDenial::PublicationNoEffect),
            PendingFate::Absent => unreachable!("the prepared original frame is retained"),
        }
    }

    /// Any refusal leaves the same original frame retained for the next driver.
    fn finish_pending(
        &self,
        medium: &dyn QueryDispatchMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<PendingFate, QueryDispatchJournalDenial> {
        let Some(frame) = self
            .journal
            .state()
            .as_ref()
            .and_then(|state| state.pending.clone())
        else {
            return Ok(PendingFate::Absent);
        };
        loop {
            match medium.advance_publication(request, &frame) {
                QueryDispatchPublicationOutcome::Committed(proof) => {
                    return self
                        .cutover(frame, proof, medium)
                        .map(PendingFate::Committed);
                }
                QueryDispatchPublicationOutcome::ProvenNoEffect(proof) => {
                    admit_subject(proof.subject(), medium, &frame)?;
                    medium.acknowledge_no_effect(proof)?;
                    self.journal
                        .state()
                        .as_mut()
                        .expect("admitted dispatch journal")
                        .pending = None;
                    return Ok(PendingFate::ProvenNoEffect);
                }
                QueryDispatchPublicationOutcome::Pending => {
                    request
                        .consult()
                        .map_err(QueryDispatchJournalDenial::Execution)?;
                    std::thread::yield_now();
                }
                QueryDispatchPublicationOutcome::Refused(cause) => return Err(cause),
                QueryDispatchPublicationOutcome::Unresolved => {
                    return Err(QueryDispatchJournalDenial::PublicationUnresolved)
                }
            }
        }
    }

    /// Durable before live: only the Home's committed witness folds the frame.
    fn cutover(
        &self,
        frame: QueryDispatchFrame,
        proof: worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryDispatch>,
        medium: &dyn QueryDispatchMedium,
    ) -> Result<QueryDispatchRecord, QueryDispatchJournalDenial> {
        proof.consume(|publication| {
            admit_subject(publication.subject(), medium, &frame)?;
            let mut guard = self.journal.state();
            let state = guard.as_mut().expect("admitted dispatch journal");
            let record = *frame.record();
            state.ledger.readmit(frame)?;
            match record {
                QueryDispatchRecord::Observation { attempt, .. } => {
                    state.unrecorded.remove(&attempt);
                }
                QueryDispatchRecord::Attempt(_) | QueryDispatchRecord::SourceControl { .. } => {}
            }
            state.pending = None;
            Ok(record)
        })
    }
}

fn admit_subject(
    subject: &worth_proof::HomeQueryDispatchSubject,
    medium: &dyn QueryDispatchMedium,
    frame: &QueryDispatchFrame,
) -> Result<(), QueryDispatchJournalDenial> {
    subject
        .admit_to(medium.authority())
        .map_err(incarnation_denial)?;
    if subject.position() != frame.position() || subject.payload_digest() != frame.payload_digest()
    {
        return Err(QueryDispatchJournalDenial::PrefixMismatch);
    }
    Ok(())
}
