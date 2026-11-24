use std::collections::BTreeMap;
use std::sync::Arc;

pub mod ancestor_record_basis;
pub mod aspect_evidence;
pub mod candidate_classification;
pub mod record_state_classification;
pub mod relation_evidence;
pub mod relation_topology;
pub mod strategy_evidence;
pub mod target_record_resolution;
pub mod visibility_evidence;

use crate::merge::conflicts::ancestor_record_basis::AncestorRecordBasisContext;
use crate::merge::conflicts::candidate_classification::classify_candidate;
use crate::merge::conflicts::relation_topology::refine_relation_topology_conflicts;
use crate::merge::data::{
    ConflictClassificationSummary, ConflictClassifiedMergePlan, IdentityScopedMergePlan,
    MergeConflictClass, MergeConflictClassification, MergePlanningError,
};

pub(super) use relation_topology::TopologyRegionWork;
use worth_relational_branch::internal_port::visibility::snapshot_read_view::RelationalReadView;
use worth_relational_contracts::internal_port::identity::data::primitives::VersionId;
use worth_relational_contracts::internal_port::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog;

pub fn classify_conflict_scope(
    aspect_plans: &AspectContractPlanCatalog,
    identity_plan: IdentityScopedMergePlan,
    base_version_id: VersionId,
    base_view: &RelationalReadView,
    target_view: &RelationalReadView,
) -> Result<(ConflictClassifiedMergePlan, TopologyRegionWork), MergePlanningError> {
    let ancestor_basis =
        AncestorRecordBasisContext::new(base_view.entities(), base_view.relations());
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

    let (classifications, topology) = refine_relation_topology_conflicts(
        &source_records_by_ref,
        &ancestor_basis,
        target_view,
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
                    aspect_plans,
                    record,
                    candidate.target_record.clone(),
                    validated_by_source.contains_key(&candidate.source_record),
                    base_version_id,
                    base_view,
                    &ancestor_basis,
                    target_view,
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

    let plan = ConflictClassifiedMergePlan {
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
    };
    Ok((plan, topology))
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
