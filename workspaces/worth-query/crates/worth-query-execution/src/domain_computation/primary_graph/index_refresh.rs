use worth_relational::facade::history::{BranchId, CommitId, RelationalCommitReceipt};
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::publication::PatchStreamRequest;

use super::WorthQueryPrimaryGraphIntegrationHandle;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrimaryGraphIndexRefreshDenialKind {
    MissingCommittedMutation,
    IndexBuildRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphIndexRefreshDenial {
    kind: WorthQueryPrimaryGraphIndexRefreshDenialKind,
    previous_commit_id: Option<CommitId>,
    committed_mutation_id: Option<CommitId>,
    committed_branch_id: Option<BranchId>,
    requested_index_count: usize,
    failed_index_count: usize,
}

impl WorthQueryPrimaryGraphIndexRefreshDenial {
    pub const fn kind(&self) -> WorthQueryPrimaryGraphIndexRefreshDenialKind {
        self.kind
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
    /// Executes one ordinary mutation and synchronously refreshes every primary
    /// identity index when that mutation advances authoritative graph state.
    ///
    /// The outer result reports derived-index maintenance. The inner result is
    /// the caller's mutation outcome and is preserved when maintenance
    /// succeeds, including ordinary mutation rejection.
    #[doc(hidden)]
    pub fn execute_mutation_with_index_refresh<T, E>(
        &self,
        mutate: impl FnOnce(&mut worth_relational::facade::runtime::RelationalRuntime) -> Result<T, E>,
    ) -> Result<Result<T, E>, WorthQueryPrimaryGraphIndexRefreshDenial> {
        self.source_owner.with_runtime_mut(|runtime| {
            let started_after = runtime
                .publication()
                .observation_snapshot()
                .latest_patch_position;
            let previous = started_after.and_then(|position| {
                runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(position)
            });
            let outcome = mutate(runtime);
            let published = runtime
                .publication()
                .read_patch_stream(PatchStreamRequest {
                    after_position: started_after,
                    max_commits: usize::MAX,
                })
                .map_err(|_| {
                    missing_committed_mutation(previous.as_ref(), self.primary_index_ids.len())
                })?;
            if published.patches.is_empty() {
                return Ok(outcome);
            }
            for patch in published.patches {
                let committed = runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(patch.position)
                    .ok_or_else(|| {
                        missing_committed_mutation(previous.as_ref(), self.primary_index_ids.len())
                    })?;
                let build = runtime
                    .index_authority()
                    .build_for_commit(DerivedIndexBuildRequest {
                        source_commit_id: committed.commit_id,
                        branch_id: committed.branch_id.clone(),
                        index_ids: self.primary_index_ids.to_vec(),
                    });
                if !build.failed_indexes.is_empty()
                    || build.generations.len() != self.primary_index_ids.len()
                {
                    return Err(index_build_rejected(
                        previous.as_ref(),
                        &committed,
                        self.primary_index_ids.len(),
                        build.failed_indexes.len(),
                    ));
                }
            }
            Ok(outcome)
        })
    }
}

fn missing_committed_mutation(
    previous: Option<&RelationalCommitReceipt>,
    requested_index_count: usize,
) -> WorthQueryPrimaryGraphIndexRefreshDenial {
    WorthQueryPrimaryGraphIndexRefreshDenial {
        kind: WorthQueryPrimaryGraphIndexRefreshDenialKind::MissingCommittedMutation,
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
        kind: WorthQueryPrimaryGraphIndexRefreshDenialKind::IndexBuildRejected,
        previous_commit_id: previous.map(|commit| commit.commit_id),
        committed_mutation_id: Some(committed.commit_id),
        committed_branch_id: Some(committed.branch_id.clone()),
        requested_index_count,
        failed_index_count,
    }
}
