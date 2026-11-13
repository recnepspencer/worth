//! One serialized original publication, with no support-state lock across IO.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;

pub(super) struct QueryObligationPublication<'a> {
    lifecycle: &'a QueryObligationHistory,
    _driver: std::sync::MutexGuard<'a, ()>,
}

impl QueryObligationHistory {
    pub(super) fn lock_publication(
        &self,
    ) -> Result<QueryObligationPublication<'_>, QueryObligationDenial> {
        match self.publication_driver.try_lock() {
            Ok(guard) => Ok(QueryObligationPublication {
                lifecycle: self,
                _driver: guard,
            }),
            Err(std::sync::TryLockError::Poisoned(cause)) => Ok(QueryObligationPublication {
                lifecycle: self,
                _driver: cause.into_inner(),
            }),
            Err(std::sync::TryLockError::WouldBlock) => {
                Err(QueryObligationDenial::PublicationPending)
            }
        }
    }
}
impl QueryObligationPublication<'_> {
    /// This carrier owns the original driver, never the token-state lock.
    pub(super) fn finish_pending(
        &self,
        medium: &dyn QueryObligationsMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        let Some(frame) = self
            .lifecycle
            .state()
            .journal
            .as_ref()
            .and_then(|journal| journal.pending.clone())
        else {
            return Ok(());
        };
        loop {
            match medium.advance_publication(request, &frame) {
                QueryObligationPublicationOutcome::Committed(proof) => {
                    return self.cutover(frame, proof, medium);
                }
                QueryObligationPublicationOutcome::ProvenNoEffect(proof) => {
                    Self::admit_subject(proof.subject(), medium, &frame)?;
                    medium.acknowledge_no_effect(proof)?;
                    let mut state = self.lifecycle.state();
                    if let Some(transfer) = state
                        .journal
                        .as_ref()
                        .and_then(|journal| journal.interpretation_transfer.as_ref())
                    {
                        transfer.validate(&state, &frame)?;
                    }
                    if let Some(transfer) = state
                        .journal
                        .as_mut()
                        .expect("original pending journal")
                        .interpretation_transfer
                        .take()
                    {
                        transfer.no_effect(&mut state);
                    }
                    state
                        .journal
                        .as_mut()
                        .expect("original pending journal")
                        .pending = None;
                    return Err(QueryObligationDenial::PublicationNoEffect);
                }
                QueryObligationPublicationOutcome::Pending => {
                    request
                        .consult()
                        .map_err(QueryObligationDenial::Execution)?;
                    std::thread::yield_now();
                }
                QueryObligationPublicationOutcome::Refused(cause) => return Err(cause),
                QueryObligationPublicationOutcome::Unresolved => {
                    return Err(QueryObligationDenial::PublicationUnresolved)
                }
            }
        }
    }
    fn admit_subject(
        subject: &worth_proof::HomeQueryObligationSubject,
        medium: &dyn QueryObligationsMedium,
        frame: &QueryObligationFrame,
    ) -> Result<(), QueryObligationDenial> {
        subject
            .admit_to(medium.authority())
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    QueryObligationDenial::ForeignHome
                }
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    QueryObligationDenial::StaleIncarnation
                }
            })?;
        if subject.position() != frame.position()
            || subject.payload_digest() != frame.payload_digest()
        {
            return Err(QueryObligationDenial::PrefixMismatch);
        }
        Ok(())
    }
    fn cutover(
        &self,
        frame: QueryObligationFrame,
        proof: worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryObligation>,
        medium: &dyn QueryObligationsMedium,
    ) -> Result<(), QueryObligationDenial> {
        proof.consume(|publication| {
            Self::admit_subject(publication.subject(), medium, &frame)?;
            let mut state = self.lifecycle.state();
            if let Some(transfer) = state
                .journal
                .as_ref()
                .and_then(|journal| journal.interpretation_transfer.as_ref())
            {
                transfer.validate(&state, &frame)?;
            }
            state
                .journal
                .as_mut()
                .expect("prepared original journal")
                .ledger
                .readmit(frame.clone())?;
            if let Some(transfer) = state
                .journal
                .as_mut()
                .expect("prepared original journal")
                .interpretation_transfer
                .take()
            {
                transfer.committed(&mut state);
            }
            if let QueryObligationRecord::ProgramSupport(QueryProgramSupportRecord::Status {
                revision,
                after,
                ..
            }) = frame.record()
                && let Some((_, entry)) = state
                    .entries
                    .iter_mut()
                    .find(|(known, _)| known.as_bytes() == revision.bytes())
            {
                entry.status = *after;
            }
            state
                .journal
                .as_mut()
                .expect("prepared original journal")
                .pending = None;
            Ok(())
        })
    }
    pub(super) fn publish(
        &self,
        medium: &dyn QueryObligationsMedium,
        record: impl Into<QueryObligationRecord>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        self.publish_record(medium, record.into(), request)
    }
    fn publish_record(
        &self,
        medium: &dyn QueryObligationsMedium,
        record: QueryObligationRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        self.finish_pending(medium, request)?;
        {
            let mut state = self.lifecycle.state();
            state
                .journal
                .as_mut()
                .ok_or(QueryObligationDenial::SupportUnavailable)?
                .prepare_pending(record)?;
        }
        self.finish_pending(medium, request)
    }
}
