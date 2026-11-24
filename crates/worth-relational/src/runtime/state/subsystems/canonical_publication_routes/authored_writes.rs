//! Discardable per-selected-prefix acceleration. No canonical envelope is retained here.
#[path = "authored_writes/adjacency.rs"]
mod adjacency;
#[path = "authored_writes/indexed.rs"]
mod indexed;
#[path = "authored_writes/managed_source.rs"]
mod managed_source;
#[path = "authored_writes/projection.rs"]
mod projection;
#[path = "authored_writes/rebuild.rs"]
mod rebuild;
use crate::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

use crate::history::access::relevant_writes::generation::AdjacencyHistoryGap;
pub(crate) use crate::history::access::relevant_writes::generation::AuthoredWriteGeneration;

impl super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn authored_generation(&self, commit: CommitId) -> Option<AuthoredWriteGeneration> {
        self.authored_generation_owner(commit)?
            .authored_write_generation()
    }

    pub(super) fn authored_generation_owner(
        &self,
        commit: CommitId,
    ) -> Option<Arc<crate::history::commit::RelationalCommitEnvelopeResidency>> {
        self.performed_envelope_residency(commit).or_else(|| {
            self.checkpoint_catalog
                .read()
                .unwrap_or_else(|p| p.into_inner())
                .get(&commit)
                .map(|artifact| artifact.envelope_residency())
        })
    }

    pub(crate) fn prepare_authored_generation(
        &self,
        execution: ExecutionRequest<'_, '_>,
        envelope: &Arc<CanonicalCommitEnvelope>,
    ) -> Result<Option<AuthoredWriteGeneration>, Denial> {
        crate::history::access::relevant_writes::run(execution, |context| {
            let mut generation = match envelope.commit.parents.as_slice() {
                [] => AuthoredWriteGeneration::default(),
                [parent] => match self.authored_generation(*parent) {
                    Some(generation) => generation,
                    None => return Ok(None),
                },
                _ => return Ok(Some(AuthoredWriteGeneration::unsupported_merge())),
            };
            extend_generation(&mut generation, execution, context, envelope)?;
            Ok(Some(generation))
        })
    }
}

pub(crate) fn commit_denial(
    cause: Denial,
) -> crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError {
    use crate::transactions::data::outcomes::commit_conflict::CommitConflict;
    use crate::transactions::data::outcomes::conflict_class::ConflictClass;
    use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial;
    use crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;
    use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError as Error;
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
        Denial::Execution(cause) => request(cause),
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
        | Denial::ForeignManagedSourceProgram
        | Denial::UnsupportedManagedSourceSelector
        | Denial::IncompleteManagedSourceHistory
        | Denial::IncompleteAdjacencyHistory { .. }
        | Denial::UnsupportedAdjacencySelector
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

fn extend_generation(
    generation: &mut AuthoredWriteGeneration,
    execution: ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    envelope: &Arc<CanonicalCommitEnvelope>,
) -> Result<(), Denial> {
    if generation.unsupported_merge {
        return Ok(());
    }
    generation.advance(execution, context, envelope.commit.commit_id)?;
    managed_source::append(generation, execution, context, envelope)?;
    indexed::append(generation, execution, context, envelope)?;
    adjacency::append(generation, execution, context, envelope)?;
    projection::append(execution, context, generation, envelope)
}
