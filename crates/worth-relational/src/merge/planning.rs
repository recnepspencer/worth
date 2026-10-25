use std::collections::BTreeMap;
use std::sync::Arc;

use crate::history::data::{BranchId, CommitId};
#[cfg(test)]
use crate::merge::data::MergePlanningRequest;
use crate::merge::data::{
    BranchCommitDelta, BranchDeltaSummary, BranchTouchedRecordDelta, HistoryScopedMergePlan,
    MergePlanningError, NormalizedRelationalMergeRequest,
};
use crate::merge::MergeAccess;
use crate::transactions::data::RecordRef;

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn plan_history_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: NormalizedRelationalMergeRequest,
    ) -> Result<HistoryScopedMergePlan, MergePlanningError> {
        crate::execution::check_request(execution).map_err(MergePlanningError::Execution)?;
        let history = self.runtime.history();
        let (target_binding, source_binding) = request
            .owner_bindings()
            .ok_or(MergePlanningError::UnboundBranchCurrentness)?;
        let basis = history
            .resolve_merge_branch_basis_from_bindings(source_binding, target_binding)
            .map_err(MergePlanningError::from)?;
        let merge_base_ancestors =
            history.ancestor_closure_by_commit_id_order(basis.merge_base.commit.commit_id);

        let target_delta_commits = self.branch_local_unique_commit_closure_by_commit_id_order(
            request.target_branch().clone(),
            basis.merge_base.supporting_left_ancestors.as_ref(),
            merge_base_ancestors.as_slice(),
        );
        let source_delta_commits = self.branch_local_unique_commit_closure_by_commit_id_order(
            request.source_branch().clone(),
            basis.merge_base.supporting_right_ancestors.as_ref(),
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
        let normalized_request = self
            .normalize_merge_planning_request(request)
            .map_err(MergePlanningError::from)?;
        self.plan_history_scope(execution, normalized_request)
    }

    fn branch_commit_delta(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch_id: crate::history::data::BranchId,
        commits: &[CommitId],
    ) -> Result<BranchCommitDelta, MergePlanningError> {
        let history = self.runtime.history();
        let mut touched_records = BTreeMap::new();
        for commit_id in commits {
            let envelope = history
                .demand_commit_envelope(execution, *commit_id)
                .map_err(MergePlanningError::Execution)?
                .ok_or(MergePlanningError::MissingDeltaCommit {
                    commit_id: *commit_id,
                })?;
            for target in envelope.touched_record_refs() {
                touched_records
                    .entry(target)
                    .or_insert_with(Vec::new)
                    .push(envelope.clone());
            }
        }

        let touched_records = touched_records
            .into_iter()
            .map(|(target, envelopes)| BranchTouchedRecordDelta {
                target,
                envelopes: Arc::from(envelopes),
            })
            .collect::<Vec<_>>();

        Ok(BranchCommitDelta {
            branch_id,
            commits: Arc::from(commits.to_vec()),
            touched_records: Arc::from(touched_records),
        })
    }

    fn branch_local_unique_commit_closure_by_commit_id_order(
        &self,
        branch_id: BranchId,
        commits: &[CommitId],
        merge_base_ancestors: &[CommitId],
    ) -> Vec<CommitId> {
        let history = self.runtime.history();
        let merge_base_ancestors = merge_base_ancestors
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        commits
            .iter()
            .copied()
            .filter(|commit_id| !merge_base_ancestors.contains(commit_id))
            .filter(|commit_id| {
                history
                    .immutable_commit_receipt(*commit_id)
                    .map(|receipt| receipt.branch_id == branch_id)
                    .unwrap_or(false)
            })
            .collect()
    }
}

pub(super) fn branch_delta_summary(
    head: &crate::history::data::RelationalCommitReceipt,
    delta: &BranchCommitDelta,
) -> BranchDeltaSummary {
    let touched_entity_count = delta
        .touched_records
        .iter()
        .filter(|record| matches!(record.target, RecordRef::Entity(_)))
        .count();
    let touched_relation_count = delta
        .touched_records
        .iter()
        .filter(|record| matches!(record.target, RecordRef::Relation(_)))
        .count();
    BranchDeltaSummary {
        branch_id: delta.branch_id.clone(),
        head_commit_id: head.commit_id,
        head_version_id: head.version_id,
        unique_commit_count: delta.commits.len(),
        touched_record_count: delta.touched_records.len(),
        touched_entity_count,
        touched_relation_count,
    }
}
