//! Opaque Query journal transport over this admitted Home's original serving.
mod publication;
use publication::{QueryObligationNativeProgress, QueryObligationPublication};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_proof::{AdmittedHomeQueryObligations, HomeQueryObligationsOwner};
use worth_query_execution::facade::program_support::*;
use worth_store::physical_runtime::*;

pub(crate) struct ApplicationQueryObligations {
    serving: Weak<ServingPhysicalRuntime>,
    home: HomeIdentity,
    incarnation: HomeIncarnation,
    owner: HomeQueryObligationsOwner,
    authority: worth_proof::HomeQueryObligationsAuthority,
    publication: Mutex<Option<Box<QueryObligationPublication>>>,
    admitted: OnceLock<AdmittedHomeQueryObligations>,
}
impl std::fmt::Debug for ApplicationQueryObligations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationQueryObligations")
            .field("home", &self.home)
            .field("incarnation", &self.incarnation)
            .finish_non_exhaustive()
    }
}
impl ApplicationQueryObligations {
    pub(crate) fn new(
        serving: &Arc<ServingPhysicalRuntime>,
        home: HomeIdentity,
        incarnation: HomeIncarnation,
    ) -> Self {
        let owner = HomeQueryObligationsOwner::fresh(home.bytes(), incarnation.fence());
        Self {
            serving: Arc::downgrade(serving),
            home,
            incarnation,
            authority: owner.authority(),
            owner,
            publication: Mutex::new(None),
            admitted: OnceLock::new(),
        }
    }
    pub(crate) fn admit(
        &self,
        proof: AdmittedHomeQueryObligations,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
        self.owner.admit(&proof).map_err(|cause| match cause {
            worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                QueryProgramSupportDenial::StaleIncarnation
            }
            worth_proof::HomeIncarnationDenial::ForeignHome => {
                QueryProgramSupportDenial::ForeignHome
            }
        })?;
        let head = self.read_journal(request, |journal| journal.head().map_err(storage))?;
        if head.map_or(0, |entry| entry.position()) != proof.position() {
            return Err(QueryProgramSupportDenial::PrefixMismatch);
        }
        if head.is_some_and(|entry| entry.payload_sha256() != proof.payload_digest()) {
            return Err(QueryProgramSupportDenial::PrefixMismatch);
        }
        self.admitted
            .set(proof)
            .map_err(|_| QueryProgramSupportDenial::SupportUnavailable)
    }
    pub(crate) fn require_admitted(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryProgramSupportDenial> {
        request
            .consult()
            .map_err(QueryProgramSupportDenial::Execution)?;
        if let Some(task) = self
            .publication
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            if !task.committed() {
                return Err(QueryProgramSupportDenial::PublicationPending);
            }
        }
        let head = self.read_journal(request, |journal| {
            let head = journal.head().map_err(storage)?;
            let position = head.map_or(0, |entry| entry.position());
            let next = position
                .checked_add(1)
                .ok_or(QueryProgramSupportDenial::IdentityExhausted)?;
            match journal.resolve(next, [0; 32]).map_err(storage)? {
                OrderedJournalResolution::NotCommitted { head: actual } if actual == position => {
                    Ok(head)
                }
                OrderedJournalResolution::Unresolved => {
                    Err(QueryProgramSupportDenial::PublicationUnresolved)
                }
                OrderedJournalResolution::NotCommitted { .. }
                | OrderedJournalResolution::Committed(_)
                | OrderedJournalResolution::Conflict(_) => {
                    Err(QueryProgramSupportDenial::PrefixMismatch)
                }
            }
        })?;
        if self.admitted.get().is_none() && head.is_some() {
            return Err(QueryProgramSupportDenial::SupportUnavailable);
        }
        Ok(())
    }
    fn read_journal<T>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        read: impl FnOnce(&OrderedJournal<'_>) -> Result<T, QueryProgramSupportDenial>,
    ) -> Result<T, QueryProgramSupportDenial> {
        let serving = self
            .serving
            .upgrade()
            .ok_or(QueryProgramSupportDenial::OwnerUnavailable)?;
        let journal = journal(&serving)?;
        let mut read = Some(read);
        let mut run = |_: Option<&worth_execution::ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|_| {
                    read.take().expect("one original read invocation")(&journal)
                })
                .map(|(result, _)| result)
                .map_err(QueryProgramSupportDenial::Execution)?
        };
        if request.is_leased() {
            request
                .in_scope(&mut run)
                .map_err(QueryProgramSupportDenial::Execution)?
        } else {
            run(None)
        }
    }
}
impl QueryObligationsMedium for ApplicationQueryObligations {
    fn authority(&self) -> &worth_proof::HomeQueryObligationsAuthority {
        &self.authority
    }
    fn home(&self) -> HomeIdentity {
        self.home
    }
    fn incarnation(&self) -> HomeIncarnation {
        self.incarnation
    }
    fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
    ) -> Result<Option<Vec<u8>>, QueryProgramSupportDenial> {
        request
            .consult()
            .map_err(QueryProgramSupportDenial::Execution)?;
        self.read_journal(request, |journal| {
            let head = journal
                .head()
                .map_err(storage)?
                .map_or(0, |row| row.position());
            if position > head {
                return match journal.resolve(position, [0; 32]).map_err(storage)? {
                    OrderedJournalResolution::NotCommitted { head: actual } if actual == head => {
                        Ok(None)
                    }
                    OrderedJournalResolution::Unresolved => {
                        Err(QueryProgramSupportDenial::PublicationUnresolved)
                    }
                    OrderedJournalResolution::NotCommitted { .. }
                    | OrderedJournalResolution::Committed(_)
                    | OrderedJournalResolution::Conflict(_) => {
                        Err(QueryProgramSupportDenial::PrefixMismatch)
                    }
                };
            }
            journal
                .read(position, QUERY_PROGRAM_SUPPORT_FRAME_MAX_BYTES)
                .map(|payload| Some(payload.into_bytes()))
                .map_err(storage)
        })
    }

    fn advance_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
        frame: &QueryProgramSupportFrame,
    ) -> QueryObligationPublicationOutcome {
        if frame.home() != self.home {
            return QueryObligationPublicationOutcome::Refused(
                QueryProgramSupportDenial::ForeignHome,
            );
        }
        let Some(serving) = self.serving.upgrade() else {
            return QueryObligationPublicationOutcome::Refused(
                QueryProgramSupportDenial::OwnerUnavailable,
            );
        };
        let mut slot = match self.publication.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return QueryObligationPublicationOutcome::Pending
            }
        };
        if slot.as_ref().is_some_and(|task| !task.matches_frame(frame)) {
            if !slot.as_ref().is_some_and(|task| task.committed()) {
                return QueryObligationPublicationOutcome::Refused(
                    QueryProgramSupportDenial::PublicationPending,
                );
            }
            *slot = None;
        }
        if slot.is_none() {
            match QueryObligationPublication::prepare(&serving, frame, request) {
                Ok(task) => *slot = Some(Box::new(task)),
                Err(cause) => return QueryObligationPublicationOutcome::Refused(cause),
            }
        }
        match slot
            .as_mut()
            .expect("same original task retained")
            .advance(&serving, request)
        {
            QueryObligationNativeProgress::Pending => QueryObligationPublicationOutcome::Pending,
            QueryObligationNativeProgress::Committed { position, digest } => {
                QueryObligationPublicationOutcome::Committed(
                    self.owner
                        .acknowledge_query_obligation_commit(position, digest),
                )
            }
            QueryObligationNativeProgress::ProvenNoEffect { position, digest } => {
                QueryObligationPublicationOutcome::ProvenNoEffect(
                    self.owner
                        .acknowledge_query_obligation_no_effect(position, digest),
                )
            }
            QueryObligationNativeProgress::Refused(cause) => {
                QueryObligationPublicationOutcome::Refused(cause)
            }
            QueryObligationNativeProgress::Unresolved => {
                QueryObligationPublicationOutcome::Unresolved
            }
        }
    }
    fn acknowledge_no_effect(
        &self,
        proof: worth_proof::NoEffectHomeQueryObligation,
    ) -> Result<(), QueryProgramSupportDenial> {
        proof
            .subject()
            .admit_to(&self.authority)
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => {
                    QueryProgramSupportDenial::ForeignHome
                }
                worth_proof::HomeIncarnationDenial::StaleIncarnation => {
                    QueryProgramSupportDenial::StaleIncarnation
                }
            })?;
        let mut slot = match self.publication.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(QueryProgramSupportDenial::PublicationPending)
            }
        };
        let task = slot
            .as_ref()
            .ok_or(QueryProgramSupportDenial::PrefixMismatch)?;
        if !task.matches_subject(proof.subject()) {
            return Err(QueryProgramSupportDenial::PrefixMismatch);
        }
        if !task.no_effect() {
            return Err(QueryProgramSupportDenial::PublicationPending);
        }
        *slot = None;
        Ok(())
    }
}
fn journal(
    serving: &ServingPhysicalRuntime,
) -> Result<OrderedJournal<'_>, QueryProgramSupportDenial> {
    serving
        .ordered_journal(
            OrderedJournalName::new("query.obligations").expect("registered Query journal"),
        )
        .map_err(storage)
}
fn storage(cause: impl std::fmt::Debug) -> QueryProgramSupportDenial {
    QueryProgramSupportDenial::Storage(format!("{cause:?}"))
}
