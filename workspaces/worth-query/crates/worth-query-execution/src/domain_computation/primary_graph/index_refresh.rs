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
    /// Executes a mutation and synchronously refreshes every committed branch
    /// touched by it. A main-branch commit uses its exact captured prior root;
    /// other branches and additional commits use the finite cold budget.
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
            let before = runtime
                .observe_branch(&runtime.main_branch_identity())
                .ok()
                .and_then(|(_, basis)| runtime.snapshots().snapshot_for_observation(&basis.observation()).ok());
            let started_after = runtime
                .publication()
                .observation_snapshot()
                .latest_patch_position;
            let previous = started_after.and_then(|position| {
                runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(position)
            });
            let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| mutate(runtime))) {
                Ok(outcome) => outcome,
                Err(payload) => {
                    if let Some(before) = before {
                        runtime.snapshots().release_snapshot(&before)
                            .expect("captured index snapshot closes during unwind");
                    }
                    std::panic::resume_unwind(payload);
                }
            };
            let published = runtime
                .publication()
                .read_patch_stream(PatchStreamRequest {
                    after_position: started_after,
                    max_commits: usize::MAX,
                });
            let published = match published {
                Ok(published) => published,
                Err(_) => {
                    if let Some(before) = before {
                        runtime.snapshots().release_snapshot(&before).expect("captured snapshot closes");
                    }
                    return Err(missing_committed_mutation(previous.as_ref(), self.primary_index_ids.len()));
                }
            };
            if published.patches.is_empty() {
                if let Some(before) = before {
                    runtime.snapshots().release_snapshot(&before).expect("captured snapshot closes");
                }
                return Ok(outcome);
            }
            let captured_branch_commit_count = published.patches.iter().filter(|patch| {
                runtime.history().immutable_commit_receipt_at_patch_stream_position(patch.position)
                    .is_some_and(|receipt| before.as_ref().is_some_and(|snapshot| receipt.branch_id == *snapshot.branch_id()))
            }).count();
            let mut used_before = false;
            let refresh = (|| {
            for patch in published.patches {
                let committed = runtime
                    .history()
                    .immutable_commit_receipt_at_patch_stream_position(patch.position)
                    .ok_or_else(|| {
                        missing_committed_mutation(previous.as_ref(), self.primary_index_ids.len())
                    })?;
                let basis = runtime.branch_identity(&committed.branch_id).ok()
                    .and_then(|identity| runtime.observe_branch(&identity).ok().map(|(_, basis)| basis))
                    .filter(|basis| basis.observation().commit_id() == Some(committed.commit_id));
                let prior = if captured_branch_commit_count == 1 && !used_before && before.as_ref().is_some_and(|before| before.branch_id() == &committed.branch_id) {
                    used_before = true;
                    before.as_ref()
                } else { None };
                let request = DerivedIndexBuildRequest {
                        source_commit_id: committed.commit_id,
                        branch_id: committed.branch_id.clone(),
                        index_ids: self.primary_index_ids.to_vec(),
                    };
                let build = if let Some(basis) = basis {
                    super::index_maintenance_budget::refresh_with_cold_fallback(runtime, request, &basis, prior)
                } else {
                    runtime.index_authority().reconstruct_for_commit(
                        request, super::index_maintenance_budget::cold_index_reconstruction_budget())
                };
                if !matches!(&build, Ok(outcome) if outcome.generations.len() == self.primary_index_ids.len()) {
                    return Err(index_build_rejected(
                        previous.as_ref(),
                        &committed,
                        self.primary_index_ids.len(),
                        self.primary_index_ids.len(),
                    ));
                }
            }
            Ok(outcome)
            })();
            if let Some(before) = before {
                runtime.snapshots().release_snapshot(&before)
                    .expect("captured index maintenance snapshot releases exactly once");
            }
            refresh
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
