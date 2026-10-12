use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::history::data::CommitId;
use crate::merge::data::{
    BranchCausalDot, CausalAnnotationSummary, CausallyAnnotatedMergePlan,
    ConflictClassifiedMergePlan, MergeCausalEvidenceModel, MergePlanningError,
    MergeRecordCausalAnnotation, MergeRecordCausalDisposition, NormalizedRelationalMergeRequest,
};
use crate::merge::MergeAccess;
use crate::transactions::data::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TouchedRecordKey {
    EntitySlot(crate::identity::data::PartitionId, u64),
    RelationSlot(crate::identity::data::PartitionId, u64),
}

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn plan_causal_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: NormalizedRelationalMergeRequest,
    ) -> Result<CausallyAnnotatedMergePlan, MergePlanningError> {
        let conflict_plan = self.plan_conflict_scope(execution, request)?;
        Ok(self.annotate_causal_scope(conflict_plan))
    }

    fn annotate_causal_scope(
        &self,
        conflict_plan: ConflictClassifiedMergePlan,
    ) -> CausallyAnnotatedMergePlan {
        let history = self.runtime.history();
        let source_touch_index = touched_record_latest_commit_index(&conflict_plan.source_delta);
        let target_touch_index = touched_record_latest_commit_index(&conflict_plan.target_delta);
        let relevant_commit_ids = source_touch_index
            .values()
            .chain(target_touch_index.values())
            .copied()
            .collect::<BTreeSet<_>>();
        let ancestor_cache = relevant_commit_ids
            .into_iter()
            .map(|commit_id| {
                (
                    commit_id,
                    history.ancestor_closure_by_commit_id_order(commit_id),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let annotations = conflict_plan
            .classifications
            .iter()
            .map(|classification| {
                let source_latest_touch = source_touch_index
                    .get(&touched_record_key(&classification.record))
                    .copied()
                    .map(|commit_id| BranchCausalDot {
                        branch_id: conflict_plan.source_delta.branch_id.clone(),
                        commit_id,
                    });
                let target_latest_touch = classification
                    .target_record
                    .as_ref()
                    .map(touched_record_key)
                    .or_else(|| {
                        classification
                            .target_record_visible
                            .then(|| touched_record_key(&classification.record))
                    })
                    .and_then(|record_key| target_touch_index.get(&record_key).copied())
                    .map(|commit_id| BranchCausalDot {
                        branch_id: conflict_plan.target_delta.branch_id.clone(),
                        commit_id,
                    });
                let disposition = causal_disposition(
                    source_latest_touch.as_ref().map(|dot| dot.commit_id),
                    target_latest_touch.as_ref().map(|dot| dot.commit_id),
                    &ancestor_cache,
                );

                MergeRecordCausalAnnotation {
                    record: classification.record.clone(),
                    target_record: classification.target_record.clone(),
                    merge_base_commit_id: conflict_plan.basis.merge_base.commit.commit_id,
                    source_latest_touch,
                    target_latest_touch,
                    disposition,
                    evidence_model: MergeCausalEvidenceModel::BranchHistoryDerived,
                }
            })
            .collect::<Vec<_>>();
        let causal_summary = summarize_causal_annotations(Arc::from(annotations.clone()));

        CausallyAnnotatedMergePlan {
            request: conflict_plan.request,
            basis: conflict_plan.basis,
            ancestry: conflict_plan.ancestry,
            target_delta: conflict_plan.target_delta,
            source_delta: conflict_plan.source_delta,
            effective_identity_declarations: conflict_plan.effective_identity_declarations,
            source_records: conflict_plan.source_records,
            candidates: conflict_plan.candidates,
            validated_schema_correspondences: conflict_plan.validated_schema_correspondences,
            identity_summary: conflict_plan.identity_summary,
            classifications: conflict_plan.classifications,
            conflict_summary: conflict_plan.conflict_summary,
            causal_annotations: Arc::from(annotations),
            causal_summary,
        }
    }
}

fn touched_record_latest_commit_index(
    delta: &crate::merge::data::BranchCommitDelta,
) -> BTreeMap<TouchedRecordKey, CommitId> {
    delta
        .touched_records
        .iter()
        .filter_map(|record| {
            record
                .commit_ids
                .last()
                .copied()
                .map(|commit_id| (touched_record_key(&record.target), commit_id))
        })
        .collect()
}

fn touched_record_key(record: &RecordRef) -> TouchedRecordKey {
    match record {
        RecordRef::Entity(entity_id) => {
            TouchedRecordKey::EntitySlot(entity_id.partition_id, entity_id.local_slot.0)
        }
        RecordRef::Relation(relation_id) => {
            TouchedRecordKey::RelationSlot(relation_id.partition_id, relation_id.local_slot.0)
        }
    }
}

fn causal_disposition(
    source_commit: Option<CommitId>,
    target_commit: Option<CommitId>,
    ancestor_cache: &BTreeMap<CommitId, Vec<CommitId>>,
) -> MergeRecordCausalDisposition {
    match (source_commit, target_commit) {
        (Some(_), None) => MergeRecordCausalDisposition::SourceOnly,
        (None, Some(_)) => MergeRecordCausalDisposition::TargetOnly,
        (None, None) => MergeRecordCausalDisposition::Equal,
        (Some(source_commit), Some(target_commit)) if source_commit == target_commit => {
            MergeRecordCausalDisposition::Equal
        }
        (Some(source_commit), Some(target_commit)) => {
            let source_ancestors = ancestor_cache
                .get(&source_commit)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let target_ancestors = ancestor_cache
                .get(&target_commit)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if target_ancestors.contains(&source_commit) {
                MergeRecordCausalDisposition::SourceBeforeTarget
            } else if source_ancestors.contains(&target_commit) {
                MergeRecordCausalDisposition::SourceAfterTarget
            } else {
                MergeRecordCausalDisposition::Concurrent
            }
        }
    }
}

fn summarize_causal_annotations(
    annotations: Arc<[MergeRecordCausalAnnotation]>,
) -> CausalAnnotationSummary {
    let mut source_only_count = 0;
    let mut target_only_count = 0;
    let mut equal_count = 0;
    let mut source_before_target_count = 0;
    let mut source_after_target_count = 0;
    let mut concurrent_count = 0;

    for annotation in annotations.iter() {
        match annotation.disposition {
            MergeRecordCausalDisposition::SourceOnly => source_only_count += 1,
            MergeRecordCausalDisposition::TargetOnly => target_only_count += 1,
            MergeRecordCausalDisposition::Equal => equal_count += 1,
            MergeRecordCausalDisposition::SourceBeforeTarget => source_before_target_count += 1,
            MergeRecordCausalDisposition::SourceAfterTarget => source_after_target_count += 1,
            MergeRecordCausalDisposition::Concurrent => concurrent_count += 1,
        }
    }

    CausalAnnotationSummary {
        classified_record_count: annotations.len(),
        source_only_count,
        target_only_count,
        equal_count,
        source_before_target_count,
        source_after_target_count,
        concurrent_count,
        evidence_model: MergeCausalEvidenceModel::BranchHistoryDerived,
        annotations,
    }
}
