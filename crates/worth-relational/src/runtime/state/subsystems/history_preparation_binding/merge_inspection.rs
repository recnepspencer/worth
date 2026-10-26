//! Merge-parent conflict discovery demands every metadata-selected envelope.
use super::RelationalPreparationHistory;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::{CommitId, MergeConflictRecord, MergeInspection};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use std::collections::BTreeSet;

impl RelationalPreparationHistory {
    pub(crate) fn demand_merge_inspection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &AdmittedRelationalBranchBasis,
        target: &AdmittedRelationalBranchBasis,
    ) -> Result<Option<MergeInspection>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let Some(source_head) = source.observation().commit_id() else {
            return Ok(None);
        };
        let Some(target_head) = target.observation().commit_id() else {
            return Ok(None);
        };
        let Some(source_receipt) = self.canonical_routes.indexed_receipt(source_head) else {
            return Ok(None);
        };
        let Some(target_receipt) = self.canonical_routes.indexed_receipt(target_head) else {
            return Ok(None);
        };
        let source_ancestors = self.ancestor_set(execution, source_head)?;
        let target_ancestors = self.ancestor_set(execution, target_head)?;
        let merge_base = source_ancestors
            .intersection(&target_ancestors)
            .copied()
            .max_by_key(|id| id.0);
        let base_ancestors = match merge_base {
            Some(base) => self.ancestor_set(execution, base)?,
            None => BTreeSet::new(),
        };
        let source_only = source_ancestors
            .difference(&base_ancestors)
            .copied()
            .collect::<Vec<_>>();
        let target_only = target_ancestors
            .difference(&base_ancestors)
            .copied()
            .collect::<Vec<_>>();
        let Some(source_records) = self.demand_commit_record_set(execution, &source_only)? else {
            return Ok(None);
        };
        let Some(target_records) = self.demand_commit_record_set(execution, &target_only)? else {
            return Ok(None);
        };
        let conflicting_records = source_records
            .intersection(&target_records)
            .cloned()
            .collect::<Vec<_>>();
        Ok(Some(MergeInspection {
            source_branch: source.identity().branch_id().clone(),
            target_branch: target.identity().branch_id().clone(),
            source_head: Some(source_receipt),
            target_head: Some(target_receipt),
            merge_base,
            source_only_commits: source_only,
            target_only_commits: target_only,
            can_merge: merge_base.is_some() && conflicting_records.is_empty(),
            conflicting_records,
        }))
    }
    fn ancestor_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        start: CommitId,
    ) -> Result<BTreeSet<CommitId>, Failure> {
        let mut found = BTreeSet::new();
        let mut pending = vec![start];
        while let Some(commit) = pending.pop() {
            crate::execution::check_request(execution).map_err(Failure::Request)?;
            if found.insert(commit)
                && let Some(receipt) = self.canonical_routes.indexed_receipt(commit)
            {
                pending.extend(receipt.parents);
            }
        }
        Ok(found)
    }
    fn demand_commit_record_set(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commits: &[CommitId],
    ) -> Result<Option<BTreeSet<MergeConflictRecord>>, Failure> {
        let mut records = BTreeSet::new();
        for commit in commits {
            let Some(envelope) = self.canonical_routes.demand_envelope(execution, *commit)? else {
                return Ok(None);
            };
            records.extend(
                envelope
                    .touched_record_refs()
                    .into_iter()
                    .map(|record| match record {
                        crate::transactions::data::RecordRef::Entity(id) => {
                            MergeConflictRecord::Entity(id)
                        }
                        crate::transactions::data::RecordRef::Relation(id) => {
                            MergeConflictRecord::Relation(id)
                        }
                    }),
            );
        }
        Ok(Some(records))
    }
}
