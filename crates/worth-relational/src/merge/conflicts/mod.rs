use std::collections::BTreeMap;
use std::sync::Arc;

mod ancestor_record_basis;
mod aspect_evidence;
mod candidate_classification;
mod record_state_classification;
mod relation_evidence;
mod relation_topology;
mod strategy_evidence;
mod target_record_resolution;
mod visibility_evidence;

use crate::merge::conflicts::ancestor_record_basis::AncestorRecordBasisContext;
use crate::merge::conflicts::candidate_classification::classify_candidate;
use crate::merge::conflicts::relation_topology::refine_relation_topology_conflicts;
use crate::merge::data::{
    ConflictClassificationSummary, ConflictClassifiedMergePlan, IdentityScopedMergePlan,
    MergeConflictClass, MergeConflictClassification, MergePlanningError,
    NormalizedRelationalMergeRequest,
};
use crate::merge::MergeAccess;

impl<'runtime> MergeAccess<'runtime> {
    pub(crate) fn plan_conflict_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: NormalizedRelationalMergeRequest,
    ) -> Result<ConflictClassifiedMergePlan, MergePlanningError> {
        let identity_plan = self.plan_identity_scope(execution, request)?;
        self.classify_conflict_scope(execution, identity_plan)
    }

    fn classify_conflict_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity_plan: IdentityScopedMergePlan,
    ) -> Result<ConflictClassifiedMergePlan, MergePlanningError> {
        let target_view = self
            .runtime
            .read_truth()
            .read_version(execution, identity_plan.basis.target_head.version_id);
        let history = self.runtime.history();
        let base_envelope = history
            .commit_envelope(identity_plan.basis.merge_base.commit.commit_id)
            .ok_or(MergePlanningError::MissingMergeBaseEnvelope {
                commit_id: identity_plan.basis.merge_base.commit.commit_id,
            })?;
        let base_version_id = base_envelope.commit.version_id;
        let base_view = self
            .runtime
            .read_truth()
            .read_version(execution, base_version_id);
        let ancestor_basis = AncestorRecordBasisContext::new(&base_view);
        let source_records_by_ref = identity_plan
            .source_records
            .iter()
            .map(|record| (record.record_ref.clone(), record))
            .collect::<BTreeMap<_, _>>();
        let validated_by_source = identity_plan
            .validated_schema_correspondences
            .iter()
            .map(|correspondence| (correspondence.source_record.clone(), correspondence))
            .collect::<BTreeMap<_, _>>();
        let source_touched_by_record = identity_plan
            .source_delta
            .touched_records
            .iter()
            .map(|delta| (delta.target.clone(), delta))
            .collect::<BTreeMap<_, _>>();
        let target_touched_by_record = identity_plan
            .target_delta
            .touched_records
            .iter()
            .map(|delta| (delta.target.clone(), delta))
            .collect::<BTreeMap<_, _>>();

        let classifications = refine_relation_topology_conflicts(
            self.runtime,
            &source_records_by_ref,
            &ancestor_basis,
            &target_view,
            identity_plan
                .candidates
                .iter()
                .map(|candidate| {
                    let record = source_records_by_ref
                        .get(&candidate.source_record)
                        .ok_or_else(|| MergePlanningError::MissingConflictSourceRecord {
                            record: candidate.source_record.clone(),
                        })?;
                    Ok(classify_candidate(
                        self.runtime,
                        record,
                        candidate.target_record.clone(),
                        validated_by_source.contains_key(&candidate.source_record),
                        base_version_id,
                        &base_view,
                        &ancestor_basis,
                        &target_view,
                        &source_touched_by_record,
                        &target_touched_by_record,
                        candidate.match_class.clone(),
                        candidate.reason.clone(),
                    ))
                })
                .collect::<Result<Vec<_>, MergePlanningError>>()?,
        );
        let classifications = Arc::<[MergeConflictClassification]>::from(classifications);
        let conflict_summary = summarize_classifications(classifications.clone());

        Ok(ConflictClassifiedMergePlan {
            request: identity_plan.request,
            basis: identity_plan.basis,
            ancestry: identity_plan.ancestry,
            target_delta: identity_plan.target_delta,
            source_delta: identity_plan.source_delta,
            effective_identity_declarations: identity_plan.effective_identity_declarations,
            source_records: identity_plan.source_records,
            candidates: identity_plan.candidates,
            validated_schema_correspondences: identity_plan.validated_schema_correspondences,
            identity_summary: identity_plan.identity_summary,
            classifications,
            conflict_summary,
        })
    }
}

fn summarize_classifications(
    classifications: Arc<[MergeConflictClassification]>,
) -> ConflictClassificationSummary {
    let mut exact_shared_truth_count = 0;
    let mut source_only_addition_count = 0;
    let mut schema_declared_correspondence_count = 0;
    let mut deletion_conflict_count = 0;
    let mut divergent_visible_state_count = 0;
    let mut strategy_intent_conflict_count = 0;
    let mut relation_endpoint_divergence_count = 0;

    for classification in classifications.iter() {
        match classification.class {
            MergeConflictClass::ExactSharedTruth => exact_shared_truth_count += 1,
            MergeConflictClass::SourceOnlyAddition => source_only_addition_count += 1,
            MergeConflictClass::SchemaDeclaredCorrespondence => {
                schema_declared_correspondence_count += 1
            }
            MergeConflictClass::Deletion(_) => deletion_conflict_count += 1,
            MergeConflictClass::DivergentVisibleState => divergent_visible_state_count += 1,
            MergeConflictClass::StrategyIntentConflict => strategy_intent_conflict_count += 1,
            MergeConflictClass::RelationEndpointDivergence => {
                relation_endpoint_divergence_count += 1
            }
        }
    }

    ConflictClassificationSummary {
        classified_record_count: classifications.len(),
        exact_shared_truth_count,
        source_only_addition_count,
        schema_declared_correspondence_count,
        deletion_conflict_count,
        divergent_visible_state_count,
        strategy_intent_conflict_count,
        relation_endpoint_divergence_count,
        classifications,
    }
}
