//! The narrow owned operation the actual Home lends Host for Query's dispatch journal.
use super::*;

/// A Home-issued handle over exactly one journal. Every handle shares the
/// Home's one journal owner and its one original publication slot, so a second
/// handle adds no second writer. It names no other journal, exposes no
/// physical runtime, and never keeps the Home's serving alive: after the
/// issuing Home releases its serving it answers `OwnerUnavailable`.
#[derive(Clone, Debug)]
pub struct ApplicationHomeDispatchJournalOperation {
    journal: Arc<ApplicationHomeDispatchJournal>,
}
impl ApplicationHomeDispatchJournalOperation {
    pub(crate) fn issue(journal: &Arc<ApplicationHomeDispatchJournal>) -> Self {
        Self {
            journal: Arc::clone(journal),
        }
    }
    pub fn authority(&self) -> &worth_proof::HomeQueryDispatchAuthority {
        &self.journal.authority
    }
    pub fn home(&self) -> HomeIdentity {
        self.journal.home
    }
    pub fn incarnation(&self) -> HomeIncarnation {
        self.journal.incarnation
    }
    /// `None` proves the position is the unwritten successor of the durable head.
    pub fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        maximum_bytes: u64,
    ) -> Result<Option<Vec<u8>>, ApplicationHomeDispatchJournalDenial> {
        use ApplicationHomeDispatchJournalDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        self.journal.read_journal(request, |journal| {
            let head = journal
                .head()
                .map_err(storage)?
                .map_or(0, |row| row.position());
            if position > head {
                if position.checked_sub(1) != Some(head) {
                    return Err(Denial::PrefixMismatch);
                }
                return require_next_uncommitted(journal, head).map(|()| None);
            }
            journal
                .read(position, maximum_bytes)
                .map(|payload| Some(payload.into_bytes()))
                .map_err(storage)
        })
    }
    /// Drive the one original publication of `frame` at `position`. The same
    /// original task stays in this Home across refusals and invoking requests.
    pub fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        frame: &[u8],
    ) -> ApplicationHomeDispatchPublication {
        use ApplicationHomeDispatchJournalDenial as Denial;
        use ApplicationHomeDispatchPublication as Outcome;
        let Some(serving) = self.journal.serving.upgrade() else {
            return Outcome::Refused(Denial::OwnerUnavailable);
        };
        let mut slot = match self.journal.publication.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Outcome::Pending,
        };
        if slot
            .as_ref()
            .is_some_and(|task| !task.matches_frame(position, frame))
        {
            if !slot.as_ref().is_some_and(|task| task.committed()) {
                return Outcome::Refused(Denial::PublicationPending);
            }
            *slot = None;
        }
        if slot.is_none() {
            match HomeJournalFramePublication::prepare(&serving, JOURNAL, position, frame, request)
            {
                Ok(task) => *slot = Some(Box::new(task)),
                Err(cause) => return Outcome::Refused(frame_denial(cause)),
            }
        }
        match slot
            .as_mut()
            .expect("same original task retained")
            .advance(&serving, request)
        {
            HomeJournalFrameProgress::Pending => Outcome::Pending,
            HomeJournalFrameProgress::Committed { position, digest } => Outcome::Committed(
                self.journal
                    .owner
                    .acknowledge_query_dispatch_commit(position, digest),
            ),
            HomeJournalFrameProgress::ProvenNoEffect { position, digest } => {
                Outcome::ProvenNoEffect(
                    self.journal
                        .owner
                        .acknowledge_query_dispatch_no_effect(position, digest),
                )
            }
            HomeJournalFrameProgress::Refused(cause) => Outcome::Refused(frame_denial(cause)),
            HomeJournalFrameProgress::Unresolved => Outcome::Unresolved,
        }
    }
    /// Release the original task only for its own proven no-effect witness.
    pub fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryDispatch,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        use ApplicationHomeDispatchJournalDenial as Denial;
        proof
            .subject()
            .admit_to(&self.journal.authority)
            .map_err(incarnation_denial)?;
        let mut slot = match self.journal.publication.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Err(Denial::PublicationPending),
        };
        let task = slot.as_ref().ok_or(Denial::PrefixMismatch)?;
        if !task.matches_subject(proof.subject().position(), proof.subject().payload_digest()) {
            return Err(Denial::PrefixMismatch);
        }
        if !task.no_effect() {
            return Err(Denial::PublicationPending);
        }
        *slot = None;
        Ok(())
    }
}
