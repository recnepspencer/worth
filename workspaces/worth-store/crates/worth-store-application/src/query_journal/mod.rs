//! Store owns one original opaque Query journal endpoint through its terminal.
mod publication;
use publication::{QueryJournalNativeProgress, QueryJournalPublication};
use std::sync::{Arc, Mutex, Weak};
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::*;

#[derive(Debug)]
pub(crate) enum QueryJournalDenial {
    PrefixMismatch,
    IdentityExhausted,
    OwnerUnavailable,
    Unavailable,
    AdmissionRequired,
    PublicationPending,
    PublicationUnresolved,
    Execution(worth_execution::WorkCeilingDenial),
    Storage(String),
}
pub(crate) enum QueryJournalPublicationOutcome {
    Pending,
    Committed { position: u64, digest: [u8; 32] },
    ProvenNoEffect { position: u64, digest: [u8; 32] },
    Refused(QueryJournalDenial),
    Unresolved,
}
pub(crate) struct ApplicationQueryJournal {
    serving: Weak<ServingPhysicalRuntime>,
    name: &'static str,
    publication: Mutex<Option<Box<QueryJournalPublication>>>,
}
impl ApplicationQueryJournal {
    pub(crate) fn new(serving: &Arc<ServingPhysicalRuntime>, name: &'static str) -> Self {
        Self {
            serving: Arc::downgrade(serving),
            name,
            publication: Mutex::new(None),
        }
    }
    pub(crate) fn admit_prefix(
        &self,
        position: u64,
        digest: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryJournalDenial> {
        let head = self.read_journal(request, |journal| journal.head().map_err(storage))?;
        if head.map_or(0, |entry| entry.position()) != position
            || head.is_some_and(|entry| entry.payload_sha256() != digest)
        {
            return Err(QueryJournalDenial::PrefixMismatch);
        }
        Ok(())
    }
    pub(crate) fn require_admitted(
        &self,
        admitted: bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryJournalDenial> {
        request.consult().map_err(QueryJournalDenial::Execution)?;
        if self
            .publication
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_some_and(|task| !task.committed())
        {
            return Err(QueryJournalDenial::PublicationPending);
        }
        let head = self.read_journal(request, |journal| {
            let head = journal.head().map_err(storage)?;
            let position = head.map_or(0, |entry| entry.position());
            let next = position
                .checked_add(1)
                .ok_or(QueryJournalDenial::IdentityExhausted)?;
            match journal.resolve(next, [0; 32]).map_err(storage)? {
                OrderedJournalResolution::NotCommitted { head: actual } if actual == position => {
                    Ok(head)
                }
                OrderedJournalResolution::Unresolved => {
                    Err(QueryJournalDenial::PublicationUnresolved)
                }
                OrderedJournalResolution::NotCommitted { .. }
                | OrderedJournalResolution::Committed(_)
                | OrderedJournalResolution::Conflict(_) => Err(QueryJournalDenial::PrefixMismatch),
            }
        })?;
        if !admitted && head.is_some() {
            return Err(QueryJournalDenial::AdmissionRequired);
        }
        Ok(())
    }
    pub(crate) fn read_frame(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        maximum: u64,
    ) -> Result<Option<Vec<u8>>, QueryJournalDenial> {
        request.consult().map_err(QueryJournalDenial::Execution)?;
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
                        Err(QueryJournalDenial::PublicationUnresolved)
                    }
                    OrderedJournalResolution::NotCommitted { .. }
                    | OrderedJournalResolution::Committed(_)
                    | OrderedJournalResolution::Conflict(_) => {
                        Err(QueryJournalDenial::PrefixMismatch)
                    }
                };
            }
            journal
                .read(position, maximum)
                .map(|payload| Some(payload.into_bytes()))
                .map_err(storage)
        })
    }
    fn read_journal<T>(
        &self,
        request: &ExecutionRequest<'_, '_>,
        read: impl FnOnce(&OrderedJournal<'_>) -> Result<T, QueryJournalDenial>,
    ) -> Result<T, QueryJournalDenial> {
        let serving = self
            .serving
            .upgrade()
            .ok_or(QueryJournalDenial::OwnerUnavailable)?;
        let journal = journal(&serving, self.name)?;
        let mut read = Some(read);
        let mut run = |_: Option<&worth_execution::ExecutionResourceLease<'_>>| {
            request
                .run_native_with_checkpoint(|_| {
                    read.take().expect("one original read invocation")(&journal)
                })
                .map(|(result, _)| result)
                .map_err(QueryJournalDenial::Execution)?
        };
        if request.is_leased() {
            request
                .in_scope(&mut run)
                .map_err(QueryJournalDenial::Execution)?
        } else {
            run(None)
        }
    }
    pub(crate) fn advance(
        &self,
        request: &ExecutionRequest<'_, '_>,
        position: u64,
        bytes: &[u8],
    ) -> QueryJournalPublicationOutcome {
        use QueryJournalPublicationOutcome as Outcome;
        let Some(serving) = self.serving.upgrade() else {
            return Outcome::Refused(QueryJournalDenial::OwnerUnavailable);
        };
        let mut slot = match self.publication.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return Outcome::Pending,
        };
        if slot
            .as_ref()
            .is_some_and(|task| !task.matches_payload(position, bytes))
        {
            if !slot.as_ref().is_some_and(|task| task.committed()) {
                return Outcome::Refused(QueryJournalDenial::PublicationPending);
            }
            *slot = None;
        }
        if slot.is_none() {
            match QueryJournalPublication::prepare(&serving, self.name, position, bytes, request) {
                Ok(task) => *slot = Some(Box::new(task)),
                Err(cause) => return Outcome::Refused(cause),
            }
        }
        match slot
            .as_mut()
            .expect("same original task retained")
            .advance(&serving, request)
        {
            QueryJournalNativeProgress::Pending => Outcome::Pending,
            QueryJournalNativeProgress::Committed { position, digest } => {
                Outcome::Committed { position, digest }
            }
            QueryJournalNativeProgress::ProvenNoEffect { position, digest } => {
                Outcome::ProvenNoEffect { position, digest }
            }
            QueryJournalNativeProgress::Refused(cause) => Outcome::Refused(cause),
            QueryJournalNativeProgress::Unresolved => Outcome::Unresolved,
        }
    }
    pub(crate) fn acknowledge_no_effect(
        &self,
        position: u64,
        digest: [u8; 32],
    ) -> Result<(), QueryJournalDenial> {
        let mut slot = match self.publication.try_lock() {
            Ok(slot) => slot,
            Err(std::sync::TryLockError::Poisoned(cause)) => cause.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => {
                return Err(QueryJournalDenial::PublicationPending)
            }
        };
        let task = slot.as_ref().ok_or(QueryJournalDenial::PrefixMismatch)?;
        if !task.matches_subject(position, digest) {
            return Err(QueryJournalDenial::PrefixMismatch);
        }
        if !task.no_effect() {
            return Err(QueryJournalDenial::PublicationPending);
        }
        *slot = None;
        Ok(())
    }
}
fn journal<'a>(
    serving: &'a ServingPhysicalRuntime,
    name: &str,
) -> Result<OrderedJournal<'a>, QueryJournalDenial> {
    serving
        .ordered_journal(
            OrderedJournalName::new(name).expect("closed Store-owned Query journal name"),
        )
        .map_err(storage)
}
fn storage(cause: impl std::fmt::Debug) -> QueryJournalDenial {
    QueryJournalDenial::Storage(format!("{cause:?}"))
}
