use crate::history::data::{BranchId, CommitId};
#[cfg(test)]
use crate::merge::data::MergePlanningRequest;
use crate::merge::data::{BranchCommitDelta, HistoryScopedMergePlan, MergePlanningError};
use crate::merge::MergeAccess;

impl<'runtime> MergeAccess<'runtime> {
    pub(in crate::merge) fn plan_history_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &crate::merge::request_normalization::BoundMergeInputs,
    ) -> Result<HistoryScopedMergePlan, MergePlanningError> {
        crate::execution::request_custody::check_request(execution)
            .map_err(MergePlanningError::Execution)?;
        let history = self.runtime.history();
        let crate::merge::request_normalization::BoundMergeInputs {
            request,
            target,
            source,
        } = request;
        let basis = history
            .resolve_merge_branch_basis_from_bindings(source, target)
            .map_err(MergePlanningError::from)?;
        let merge_base_ancestors =
            history.ancestor_closure_by_commit_id_order(basis.merge_base().commit().commit_id);

        let target_delta_commits = self.branch_local_unique_commit_closure_by_commit_id_order(
            request.target_branch().clone(),
            basis.merge_base().supporting_left_ancestors(),
            merge_base_ancestors.as_slice(),
        );
        let source_delta_commits = self.branch_local_unique_commit_closure_by_commit_id_order(
            request.source_branch().clone(),
            basis.merge_base().supporting_right_ancestors(),
            merge_base_ancestors.as_slice(),
        );

        Ok(HistoryScopedMergePlan {
            request: request.clone(),
            basis,
            target_delta: self.branch_commit_delta(
                execution,
                request.target_branch().clone(),
                target_delta_commits.as_slice(),
            )?,
            source_delta: self.branch_commit_delta(
                execution,
                request.source_branch().clone(),
                source_delta_commits.as_slice(),
            )?,
        })
    }

    #[cfg(test)]
    pub(crate) fn plan_history_scope_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: MergePlanningRequest,
    ) -> Result<HistoryScopedMergePlan, MergePlanningError> {
        let bound = self.bind_planning_request_for_test(request)?;
        let normalized_request =
            super::request_normalization::normalize_bound_merge_planning_request(bound)?;
        self.plan_history_scope(execution, &normalized_request)
    }

    fn branch_commit_delta(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: crate::history::data::BranchId,
        commits: &[CommitId],
    ) -> Result<BranchCommitDelta, MergePlanningError> {
        let history = self.runtime.history();
        super::branch_delta::assemble_branch_delta(
            branch_id,
            commits,
            &mut commits.iter().map(|commit_id| {
                history
                    .demand_commit_envelope(execution, *commit_id)
                    .map_err(MergePlanningError::Execution)?
                    .ok_or(MergePlanningError::MissingDeltaCommit {
                        commit_id: *commit_id,
                    })
            }),
        )
    }

    fn branch_local_unique_commit_closure_by_commit_id_order(
        &self,
        branch_id: BranchId,
        commits: &[CommitId],
        merge_base_ancestors: &[CommitId],
    ) -> Vec<CommitId> {
        let history = self.runtime.history();
        super::branch_delta::branch_local_unique_commits(
            commits,
            merge_base_ancestors,
            &|commit_id| {
                history
                    .immutable_commit_receipt(commit_id)
                    .is_some_and(|receipt| receipt.branch_id == branch_id)
            },
        )
    }
}
