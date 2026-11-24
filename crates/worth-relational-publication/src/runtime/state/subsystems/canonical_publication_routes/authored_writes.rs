//! Discardable per-selected-prefix acceleration. No canonical envelope is retained here.
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;
#[path = "authored_writes/adjacency.rs"]
pub mod adjacency;
#[path = "authored_writes/indexed.rs"]
pub mod indexed;
#[path = "authored_writes/managed_source.rs"]
pub mod managed_source;
#[path = "authored_writes/projection.rs"]
pub mod projection;
#[path = "authored_writes/rebuild.rs"]
pub mod rebuild;
use branch::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use branch::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use contracts::history::data::coordinates::CommitId;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub use branch::history::access::relevant_writes::generation::AuthoredWriteGeneration;

impl super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn authored_generation(&self, commit: CommitId) -> Option<AuthoredWriteGeneration> {
        self.authored_generation_owner(commit)?
            .authored_write_generation()
    }

    pub(super) fn authored_generation_owner(
        &self,
        commit: CommitId,
    ) -> Option<Arc<branch::history::commit::envelope_residency::RelationalCommitEnvelopeResidency>>
    {
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
        branch::history::access::relevant_writes::run(execution, |context| {
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

pub fn commit_denial(
    cause: Denial,
) -> contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError {
    use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
    use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
    use contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial;
    use contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind;
    use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError as Error;
    let request = |cause| {
        Error::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        })
    };
    match cause {
        Denial::BackingAllocation(_) => {
            request(contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed)
        }
        Denial::Allocation(denial)
        | Denial::OrderedStorage(worth_execution::ExecutionOrderedMapDenial::Allocation(denial)) => {
            contracts::execution::denial::allocation_commit_error(Error::conflict(CommitConflict::new(
                ConflictClass::ExecutionAllocationDenied { denial },
            )))
        }
        Denial::Request(cause)
        | Denial::OrderedStorage(worth_execution::ExecutionOrderedMapDenial::Request(cause)) => {
            request(contracts::execution::denial_cause::RelationalExecutionDenialCause::from(
                cause,
            ))
        }
        Denial::Interrupted(cause) => request(contracts::execution::denial::kernel_stop(cause)),
        Denial::Execution(cause) => request(cause),
        Denial::SizeOverflow
        | Denial::OrderedStorage(worth_execution::ExecutionOrderedMapDenial::CapacityOverflow) => {
            request(contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow)
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
