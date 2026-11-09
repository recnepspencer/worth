use worth_relational::facade::history::{BranchId, CommitId, RelationalCommitReceipt};
use worth_relational::facade::publication::PatchStreamRequest;

use super::WorthQueryPrimaryGraphIntegrationHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrimaryGraphIndexRefreshDenialKind {
    MissingCommittedMutation,
    IndexBuildRejected,
    IndexPredecessorUnavailable,
    Execution(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphIndexRefreshDenial {
    kind: Box<WorthQueryPrimaryGraphIndexRefreshDenialKind>,
    previous_commit_id: Option<CommitId>,
    committed_mutation_id: Option<CommitId>,
    committed_branch_id: Option<BranchId>,
    requested_index_count: usize,
    failed_index_count: usize,
}

impl WorthQueryPrimaryGraphIndexRefreshDenial {
    pub fn kind(&self) -> WorthQueryPrimaryGraphIndexRefreshDenialKind {
        *self.kind
    }

    pub const fn previous_commit_id(&self) -> Option<CommitId> {
        self.previous_commit_id
    }

    pub const fn committed_mutation_id(&self) -> Option<CommitId> {
        self.committed_mutation_id
    }

    pub fn committed_branch_id(&self) -> Option<&BranchId> {
        self.committed_branch_id.as_ref()
    }

    pub const fn requested_index_count(&self) -> usize {
        self.requested_index_count
    }

    pub const fn failed_index_count(&self) -> usize {
        self.failed_index_count
    }
}

impl std::fmt::Display for WorthQueryPrimaryGraphIndexRefreshDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "primary graph index refresh denied: {:?} (previous={:?}, committed={:?}, branch={:?}, requested={}, failed={})",
            self.kind,
            self.previous_commit_id,
            self.committed_mutation_id,
            self.committed_branch_id,
            self.requested_index_count,
            self.failed_index_count,
        )
    }
}

impl std::error::Error for WorthQueryPrimaryGraphIndexRefreshDenial {}

impl WorthQueryPrimaryGraphIntegrationHandle {
    /// Execute a mutation and consume the native pipeline's selections at the
    /// final live heads reached by this batch. Intermediate selections may
    /// already have retired. Missing preparation at a selected head refuses;
    /// batch and non-main writes never trigger a post-publication rebuild.
    #[doc(hidden)]
    pub fn execute_mutation_with_index_refresh<T, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&worth_relational::facade::runtime::RelationalRuntime) -> Result<T, E>,
    ) -> Result<Result<T, E>, WorthQueryPrimaryGraphIndexRefreshDenial> {
        self.source_owner.with_runtime(execution, |runtime| {
            let position = runtime.history().latest_patch_stream_position();
            let previous = position.and_then(|position| {
                runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(position)
            });
            let outcome = mutate(runtime);
            let published =
                read_complete_mutation_batch(runtime, execution, position).map_err(|failure| {
                    patch_read_denial(
                        previous.as_ref(),
                        runtime.history().historical_latest_commit().as_ref(),
                        self.primary_index_ids.len(),
                        failure,
                    )
                })?;
            for patch in published.patches {
                let committed = runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(patch.position)
                    .ok_or_else(|| {
                        missing_committed_mutation(previous.as_ref(), self.primary_index_ids.len())
                    })?;
                runtime
                    .index_authority()
                    .verify_index_basis_for_commit(execution, &committed)
                    .map_err(|denial| {
                        maintenance_denial(
                            previous.as_ref(),
                            &committed,
                            self.primary_index_ids.len(),
                            denial,
                        )
                    })?;
            }
            Ok(outcome)
        })
    }
}

fn read_complete_mutation_batch(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    after_position: Option<worth_relational::facade::publication::PatchStreamPosition>,
) -> Result<
    worth_relational::facade::publication::PatchStreamBatch,
    worth_relational::facade::publication::PatchStreamReadErrorClass,
> {
    use worth_relational::facade::publication::PatchStreamReadErrorClass;
    let batch = runtime
        .publication()
        .read_patch_stream(
            execution,
            PatchStreamRequest {
                after_position,
                ..PatchStreamRequest::default()
            },
        )
        .map_err(|failure| failure.class)?;
    if !batch.patches.is_empty() && batch.next_position != batch.latest_position {
        return Err(PatchStreamReadErrorClass::ReadRequest(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::ResultCapacityExceeded,
        ));
    }
    Ok(batch)
}

pub fn patch_read_denial(
    previous: Option<&RelationalCommitReceipt>,
    latest: Option<&RelationalCommitReceipt>,
    requested: usize,
    class: worth_relational::facade::publication::PatchStreamReadErrorClass,
) -> WorthQueryPrimaryGraphIndexRefreshDenial {
    let mut refusal = missing_committed_mutation(previous, requested);
    if let Some(commit) =
        latest.filter(|commit| previous.is_none_or(|prior| prior.commit_id != commit.commit_id))
    {
        refusal.committed_mutation_id = Some(commit.commit_id);
        refusal.committed_branch_id = Some(commit.branch_id.clone());
    }
    if let worth_relational::facade::publication::PatchStreamReadErrorClass::ReadRequest(cause) =
        class
    {
        refusal.kind = Box::new(WorthQueryPrimaryGraphIndexRefreshDenialKind::Execution(
            super::provider::relational_execution_denial::relational_execution_kind(cause, None),
        ));
    }
    refusal
}

fn missing_committed_mutation(
    previous: Option<&RelationalCommitReceipt>,
    requested_index_count: usize,
) -> WorthQueryPrimaryGraphIndexRefreshDenial {
    WorthQueryPrimaryGraphIndexRefreshDenial {
        kind: Box::new(WorthQueryPrimaryGraphIndexRefreshDenialKind::MissingCommittedMutation),
        previous_commit_id: previous.map(|commit| commit.commit_id),
        committed_mutation_id: None,
        committed_branch_id: None,
        requested_index_count,
        failed_index_count: requested_index_count,
    }
}

fn index_build_rejected(
    previous: Option<&RelationalCommitReceipt>,
    committed: &RelationalCommitReceipt,
    requested_index_count: usize,
    failed_index_count: usize,
) -> WorthQueryPrimaryGraphIndexRefreshDenial {
    WorthQueryPrimaryGraphIndexRefreshDenial {
        kind: Box::new(WorthQueryPrimaryGraphIndexRefreshDenialKind::IndexBuildRejected),
        previous_commit_id: previous.map(|commit| commit.commit_id),
        committed_mutation_id: Some(committed.commit_id),
        committed_branch_id: Some(committed.branch_id.clone()),
        requested_index_count,
        failed_index_count,
    }
}

fn maintenance_denial(
    previous: Option<&RelationalCommitReceipt>,
    committed: &RelationalCommitReceipt,
    requested: usize,
    denial: worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
) -> WorthQueryPrimaryGraphIndexRefreshDenial {
    use worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind as Native;
    let mut refusal = index_build_rejected(previous, committed, requested, requested);
    match denial.kind {
        Native::Execution(cause) => {
            refusal.kind = Box::new(WorthQueryPrimaryGraphIndexRefreshDenialKind::Execution(
                super::provider::relational_execution_denial::relational_execution_kind(
                    cause, None,
                ),
            ))
        }
        Native::ColdReconstructionRequired => {
            refusal.kind =
                Box::new(WorthQueryPrimaryGraphIndexRefreshDenialKind::IndexPredecessorUnavailable);
        }
        Native::Basis(_)
        | Native::SnapshotUnavailable
        | Native::CommitMismatch
        | Native::BeforeRootMismatch
        | Native::IndexUnavailable(_)
        | Native::GenerationKindMismatch(_)
        | Native::PriorEntryMismatch
        | Native::WorkBudgetExceeded
        | Native::ForeignCandidate
        | Native::CandidateLifetimeExpired { .. }
        | Native::CandidateUnavailable
        | Native::CandidateIndexesAlreadyPrepared
        | Native::GenerationIdentityExhausted => {}
    }
    refusal
}
