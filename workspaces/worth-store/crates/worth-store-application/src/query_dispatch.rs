//! The Home's one `query.dispatch` journal operation: opaque Query frames over
//! this admitted Home's original serving. Store never decodes a frame.
use crate::journal_frame_publication::{
    HomeJournalFrameDenial, HomeJournalFrameProgress, HomeJournalFramePublication,
};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_proof::{AdmittedHomeQueryDispatch, HomeQueryDispatchOwner};
use worth_store::physical_runtime::*;
mod operation;
pub use operation::ApplicationHomeDispatchJournalOperation;

const JOURNAL: &str = "query.dispatch";

#[derive(Debug)]
pub enum ApplicationHomeDispatchJournalDenial {
    ForeignHome,
    StaleIncarnation,
    PrefixMismatch,
    IdentityExhausted,
    /// The Home that issued this operation has released its serving.
    OwnerUnavailable,
    /// A recorded prefix exists that its Query owner has not admitted.
    AdmissionUnavailable,
    PublicationPending,
    PublicationUnresolved,
    DeadlineUnavailable,
    Storage(String),
    Execution(worth_execution::WorkCeilingDenial),
}

#[derive(Debug)]
pub enum ApplicationHomeDispatchPublication {
    Pending,
    Committed(worth_proof::DurablyCommitted<worth_proof::CommittedHomeQueryDispatch>),
    ProvenNoEffect(worth_proof::NoEffectHomeQueryDispatch),
    Refused(ApplicationHomeDispatchJournalDenial),
    Unresolved,
}

pub(crate) struct ApplicationHomeDispatchJournal {
    serving: Weak<ServingPhysicalRuntime>,
    home: HomeIdentity,
    incarnation: HomeIncarnation,
    owner: HomeQueryDispatchOwner,
    authority: worth_proof::HomeQueryDispatchAuthority,
    publication: Mutex<Option<Box<HomeJournalFramePublication>>>,
    admitted: OnceLock<AdmittedHomeQueryDispatch>,
}
impl std::fmt::Debug for ApplicationHomeDispatchJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationHomeDispatchJournal")
            .field("home", &self.home)
            .field("incarnation", &self.incarnation)
            .finish_non_exhaustive()
    }
}
impl ApplicationHomeDispatchJournal {
    pub(crate) fn new(
        serving: &Arc<ServingPhysicalRuntime>,
        home: HomeIdentity,
        incarnation: HomeIncarnation,
    ) -> Self {
        let owner = HomeQueryDispatchOwner::fresh(home.bytes(), incarnation.fence());
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
    /// The Query owner's decoded prefix must be this journal's actual head.
    pub(crate) fn admit(
        &self,
        proof: AdmittedHomeQueryDispatch,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        use ApplicationHomeDispatchJournalDenial as Denial;
        self.owner.admit(&proof).map_err(incarnation_denial)?;
        let head = self.read_journal(request, |journal| journal.head().map_err(storage))?;
        if head.map_or(0, |entry| entry.position()) != proof.position() {
            return Err(Denial::PrefixMismatch);
        }
        if head.is_some_and(|entry| entry.payload_sha256() != proof.payload_digest()) {
            return Err(Denial::PrefixMismatch);
        }
        self.admitted
            .set(proof)
            .map_err(|_| Denial::AdmissionUnavailable)
    }
    /// Service requires an admitted prefix and no publication of unknown fate.
    pub(crate) fn require_admitted(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), ApplicationHomeDispatchJournalDenial> {
        use ApplicationHomeDispatchJournalDenial as Denial;
        request.consult().map_err(Denial::Execution)?;
        if let Some(task) = self
            .publication
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            if !task.committed() {
                return Err(Denial::PublicationPending);
            }
        }
        let head = self.read_journal(request, |journal| {
            let head = journal.head().map_err(storage)?;
            let position = head.map_or(0, |entry| entry.position());
            require_next_uncommitted(journal, position)?;
            Ok(head)
        })?;
        if self.admitted.get().is_none() && head.is_some() {
            return Err(Denial::AdmissionUnavailable);
        }
        Ok(())
    }
    fn read_journal<T>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        read: impl FnOnce(&OrderedJournal<'_>) -> Result<T, ApplicationHomeDispatchJournalDenial>,
    ) -> Result<T, ApplicationHomeDispatchJournalDenial> {
        use ApplicationHomeDispatchJournalDenial as Denial;
        let serving = self.serving.upgrade().ok_or(Denial::OwnerUnavailable)?;
        let journal = serving
            .ordered_journal(OrderedJournalName::new(JOURNAL).expect("registered Query journal"))
            .map_err(storage)?;
        let mut read = Some(read);
        let mut run = |_: Option<&worth_execution::ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|_| {
                    read.take().expect("one original read invocation")(&journal)
                })
                .map(|(result, _)| result)
                .map_err(Denial::Execution)?
        };
        if request.is_leased() {
            request.in_scope(&mut run).map_err(Denial::Execution)?
        } else {
            run(None)
        }
    }
}

/// The successor of `position` must be provably unwritten.
fn require_next_uncommitted(
    journal: &OrderedJournal<'_>,
    position: u64,
) -> Result<(), ApplicationHomeDispatchJournalDenial> {
    use ApplicationHomeDispatchJournalDenial as Denial;
    let next = position.checked_add(1).ok_or(Denial::IdentityExhausted)?;
    match journal.resolve(next, [0; 32]).map_err(storage)? {
        OrderedJournalResolution::NotCommitted { head } if head == position => Ok(()),
        OrderedJournalResolution::Unresolved => Err(Denial::PublicationUnresolved),
        OrderedJournalResolution::NotCommitted { .. }
        | OrderedJournalResolution::Committed(_)
        | OrderedJournalResolution::Conflict(_) => Err(Denial::PrefixMismatch),
    }
}
fn incarnation_denial(
    cause: worth_proof::HomeIncarnationDenial,
) -> ApplicationHomeDispatchJournalDenial {
    match cause {
        worth_proof::HomeIncarnationDenial::StaleIncarnation => {
            ApplicationHomeDispatchJournalDenial::StaleIncarnation
        }
        worth_proof::HomeIncarnationDenial::ForeignHome => {
            ApplicationHomeDispatchJournalDenial::ForeignHome
        }
    }
}
fn frame_denial(cause: HomeJournalFrameDenial) -> ApplicationHomeDispatchJournalDenial {
    match cause {
        HomeJournalFrameDenial::Execution(cause) => {
            ApplicationHomeDispatchJournalDenial::Execution(cause)
        }
        HomeJournalFrameDenial::DeadlineUnavailable => {
            ApplicationHomeDispatchJournalDenial::DeadlineUnavailable
        }
        HomeJournalFrameDenial::Storage(detail) => {
            ApplicationHomeDispatchJournalDenial::Storage(detail)
        }
    }
}
fn storage(cause: impl std::fmt::Debug) -> ApplicationHomeDispatchJournalDenial {
    ApplicationHomeDispatchJournalDenial::Storage(format!("{cause:?}"))
}
