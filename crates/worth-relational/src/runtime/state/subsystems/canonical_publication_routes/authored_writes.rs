//! Discardable per-selected-prefix acceleration. No canonical envelope is retained here.
#[path = "authored_writes/indexed.rs"]
mod indexed;
#[path = "authored_writes/projection.rs"]
mod projection;
#[path = "authored_writes/rebuild.rs"]
mod rebuild;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::history::{
    RelationalAuthoredWriteDenial as Denial, RelationalAuthoredWriteSelector as Key,
};
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionOrderedMap, ExecutionRequest};

#[derive(Clone, Debug, Default)]
pub(crate) struct AuthoredWriteGeneration {
    pub(crate) writes: ExecutionOrderedMap<Key, Postings>,
    pub(crate) prefix: ExecutionOrderedMap<CommitId, u64>,
    pub(crate) incomplete_indexes: ExecutionOrderedMap<u64, ()>,
    pub(crate) index_coverage: ExecutionOrderedMap<u64, crate::runtime::IndexCatalogCoverage>,
    pub(crate) index_run_start: u64,
    pub(crate) index_run_last: Option<crate::runtime::IndexCatalogCoverage>,
    pub(crate) ordinal: u64,
    pub(crate) unsupported_merge: bool,
    _funding: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Postings(pub(crate) ExecutionOrderedMap<u64, CommitId>);
impl ChargedBytes for Postings {
    // The shared cells retain their actual original allocation grants.
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn authored_generation(&self, commit: CommitId) -> Option<AuthoredWriteGeneration> {
        let route = self.by_commit.get(&commit)?;
        if !route.is_visible() {
            return None;
        }
        route
            .authored_writes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }

    pub(crate) fn prepare_authored_generation(
        &self,
        execution: ExecutionRequest<'_, '_>,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<Option<AuthoredWriteGeneration>, Denial> {
        crate::history::run_authored_write_stage(execution, |context| {
            let mut generation = match envelope.commit.parents.as_slice() {
                [] => AuthoredWriteGeneration::default(),
                [parent] => match self.authored_generation(*parent) {
                    Some(generation) => generation,
                    None => return Ok(None),
                },
                _ => {
                    return Ok(Some(AuthoredWriteGeneration {
                        unsupported_merge: true,
                        ..Default::default()
                    }))
                }
            };
            generation.extend(execution, context, envelope)?;
            Ok(Some(generation))
        })
    }
}

pub(crate) fn commit_denial(cause: Denial) -> crate::transactions::data::TransactionCommitError {
    use crate::transactions::data::{
        CommitConflict, CommitExecutionDenial, CommitExecutionDenialKind, ConflictClass,
        TransactionCommitError as Error,
    };
    let request = |cause| {
        Error::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        })
    };
    match cause {
        Denial::BackingAllocation(_) => {
            request(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
        }
        Denial::Allocation(denial)
        | Denial::OrderedStorage(worth_execution::ExecutionOrderedMapDenial::Allocation(denial)) => {
            crate::execution::allocation_commit_error(Error::conflict(CommitConflict::new(
                ConflictClass::ExecutionAllocationDenied { denial },
            )))
        }
        Denial::Request(cause)
        | Denial::OrderedStorage(worth_execution::ExecutionOrderedMapDenial::Request(cause)) => {
            request(crate::execution::RelationalExecutionDenialCause::from(
                cause,
            ))
        }
        Denial::Interrupted(cause) => request(crate::execution::denial::kernel_stop(cause)),
        Denial::SizeOverflow
        | Denial::OrderedStorage(worth_execution::ExecutionOrderedMapDenial::CapacityOverflow) => {
            request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        }
        Denial::ForeignOwner
        | Denial::NoSelectedCommit
        | Denial::NotExactSelection
        | Denial::SourceUnavailable
        | Denial::CanonicalEvidenceUnavailable
        | Denial::GenerationUnavailable
        | Denial::UnsupportedIndexSelector
        | Denial::IncompleteIndexHistory
        | Denial::UnsupportedMergeInterval
        | Denial::NotSelectedPrefix
        | Denial::EmptySelection
        | Denial::InvalidContinuation
        | Denial::PageStorageTooSmall
        | Denial::Retention(_) => {
            unreachable!("generation preparation emits only original-request/storage stops")
        }
    }
}

impl AuthoredWriteGeneration {
    fn extend(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), Denial> {
        if self.unsupported_merge {
            return Ok(());
        }
        self._funding = execution
            .reserve_retained_memory(
                (std::mem::size_of::<AuthoredWriteGeneration>()
                    + std::mem::size_of::<worth_execution::ExecutionMemoryReservation>()
                    + 2 * std::mem::size_of::<usize>()) as u64,
            )?
            .map(Arc::new);
        self.ordinal = self.ordinal.checked_add(1).ok_or(Denial::SizeOverflow)?;
        crate::history::visit_authored_write_lookup(context, self.prefix.len(), 8)?;
        self.prefix.insert_cloned_native_for_request(
            &envelope.commit.commit_id,
            &self.ordinal,
            execution,
        )?;
        indexed::append(self, execution, context, envelope)?;
        projection::append(execution, context, self, envelope)
    }
    pub(super) fn append(
        &mut self,
        execution: ExecutionRequest<'_, '_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        key: &Key,
        commit: CommitId,
    ) -> Result<(), Denial> {
        let key_work = key
            .additional_charged_bytes()
            .checked_add(std::mem::size_of::<Key>() as u64)
            .ok_or(Denial::Interrupted(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            ))?;
        crate::history::visit_authored_write_lookup(context, self.writes.len(), key_work)?;
        let mut postings = self.writes.get(key).cloned().unwrap_or_default();
        crate::history::visit_authored_write_lookup(context, postings.0.len(), 8)?;
        postings
            .0
            .insert_cloned_native_for_request(&self.ordinal, &commit, execution)?;
        crate::history::visit_authored_write_lookup(context, self.writes.len(), key_work)?;
        self.writes
            .insert_cloned_for_request(key, &postings, execution)?;
        Ok(())
    }
}
