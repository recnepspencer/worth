//! One serialized original publication, with no support-state lock across IO.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;

pub(super) struct SupportJournalPublication<'a> {
    lifecycle: &'a WorthQueryProgramSupportLifecycle,
    _driver: std::sync::MutexGuard<'a, ()>,
}

impl WorthQueryProgramSupportLifecycle {
    pub(super) fn lock_publication(
        &self,
    ) -> Result<SupportJournalPublication<'_>, QueryProgramSupportDenial> {
        match self.publication_driver.try_lock() {
            Ok(guard) => Ok(SupportJournalPublication {
                lifecycle: self,
                _driver: guard,
            }),
            Err(std::sync::TryLockError::Poisoned(cause)) => Ok(SupportJournalPublication {
                lifecycle: self,
                _driver: cause.into_inner(),
            }),
            Err(std::sync::TryLockError::WouldBlock) => {
                Err(QueryProgramSupportDenial::PublicationPending)
            }
        }
    }
}
impl SupportJournalPublication<'_> {
    /// This carrier owns the original driver, never the token-state lock.
    pub(super) fn finish_pending(
        &self,
        medium: &dyn QueryObligationsMedium,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
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
                    return Err(QueryProgramSupportDenial::PublicationNoEffect);
                }
                QueryObligationPublicationOutcome::Pending => {
                    request
                        .consult()
                        .map_err(QueryProgramSupportDenial::Execution)?;
                    std::thread::yield_now();
                }
                QueryObligationPublicationOutcome::Refused(cause) => return Err(cause),
                QueryObligationPublicationOutcome::Unresolved => {
                    return Err(QueryProgramSupportDenial::PublicationUnresolved)
                }
            }
        }
    }
    fn admit_subject(
        subject: &worth_proof::HomeQueryObligationSubject,
        medium: &dyn QueryObligationsMedium,
        frame: &QueryProgramSupportFrame,
    ) -> Result<(), QueryProgramSupportDenial> {
        subject
            .admit_to(medium.authority())
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    QueryProgramSupportDenial::ForeignHome
                }
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    QueryProgramSupportDenial::StaleIncarnation
                }
            })?;
        if subject.position() != frame.position()
            || subject.payload_digest() != frame.payload_digest()
        {
            return Err(QueryProgramSupportDenial::PrefixMismatch);
        }
        Ok(())
    }
    fn cutover(
        &self,
        frame: QueryProgramSupportFrame,
        proof: worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryObligation>,
        medium: &dyn QueryObligationsMedium,
    ) -> Result<(), QueryProgramSupportDenial> {
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
            if let QueryProgramSupportRecord::Status {
                revision, after, ..
            } = frame.record()
            {
                if let Some((_, entry)) = state
                    .entries
                    .iter_mut()
                    .find(|(known, _)| known.as_bytes() == revision.bytes())
                {
                    entry.status = *after;
                }
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
        record: QueryProgramSupportRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
        self.finish_pending(medium, request)?;
        {
            let mut state = self.lifecycle.state();
            state
                .journal
                .as_mut()
                .ok_or(QueryProgramSupportDenial::SupportUnavailable)?
                .prepare_pending(record)?;
        }
        self.finish_pending(medium, request)
    }
}
