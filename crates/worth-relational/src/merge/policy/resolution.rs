use std::collections::BTreeMap;
use std::sync::Arc;

use crate::capabilities::RuntimeConfigSource;
use crate::merge::aspect_plan_lookup::lowered_plan_for_record;
use crate::merge::data::{
    AspectPolicyResolutionRecord, CausallyAnnotatedMergePlan, MergePlanningError,
    MergePolicyProofBoundary, MergePolicyResolutionRecord, PolicyResolvedMergePlan,
};
use crate::runtime::RelationalRuntime;
use crate::storage::data::RelationalReadView;

use super::contexts::{PolicyReadViewContext, PolicyReadViewIndex};
use super::decisions::{
    aggregate_record_resolution, decision_boundary_for_aspect, effective_merge_policies_for_record,
    ownership_surface_for_policies, summarize_policy_records,
};
use super::value_strategy::{
    resolve_aspect_value_strategy, resolve_policy_aspect_value_basis, scalar_policy_aspect_binding,
    AutoResolutionStrategy, ScalarPolicyBindingDenial,
};

pub(super) fn resolve_policy_scope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    causal_plan: CausallyAnnotatedMergePlan,
) -> Result<PolicyResolvedMergePlan, MergePlanningError> {
    let history = runtime.history();
    let base_envelope = history
        .commit_envelope(causal_plan.basis.merge_base.commit.commit_id)
        .ok_or(MergePlanningError::MissingMergeBaseEnvelope {
            commit_id: causal_plan.basis.merge_base.commit.commit_id,
        })?;
    let source_view = runtime
        .read_truth()
        .read_version(execution, causal_plan.basis.source_head.version_id);
    let target_view = runtime
        .read_truth()
        .read_version(execution, causal_plan.basis.target_head.version_id);
    let base_view = runtime
        .read_truth()
        .read_version(execution, base_envelope.commit.version_id);
    let source_view_index = PolicyReadViewIndex::new(&source_view);
    let target_view_index = PolicyReadViewIndex::new(&target_view);
    let base_view_index = PolicyReadViewIndex::new(&base_view);
    let schema_registry = &runtime.runtime_config().schema.registry;
    let source_view_context = PolicyReadViewContext::new(&source_view, &source_view_index);
    let target_view_context = PolicyReadViewContext::new(&target_view, &target_view_index);
    let source_records_by_ref = causal_plan
        .source_records
        .iter()
        .map(|record| (record.record_ref.clone(), record))
        .collect::<BTreeMap<_, _>>();
    let causal_dispositions_by_record = causal_plan
        .causal_annotations
        .iter()
        .map(|annotation| (annotation.record.clone(), annotation.disposition))
        .collect::<BTreeMap<_, _>>();
    let causal_annotations_by_record = causal_plan
        .causal_annotations
        .iter()
        .map(|annotation| (annotation.record.clone(), annotation))
        .collect::<BTreeMap<_, _>>();
    let mut record_base_views =
        BTreeMap::<crate::history::data::CommitId, RelationalReadView>::new();
    let mut record_base_view_indices =
        BTreeMap::<crate::history::data::CommitId, PolicyReadViewIndex>::new();
    let policy_records = causal_plan
        .classifications
        .iter()
        .map(|classification| {
            let record = source_records_by_ref
                .get(&classification.record)
                .ok_or_else(|| MergePlanningError::MissingPolicySourceRecord {
                    record: classification.record.clone(),
                })?;
            let applied_policies = effective_merge_policies_for_record(schema_registry, record);
            let annotation = causal_annotations_by_record
                .get(&classification.record)
                .ok_or_else(|| MergePlanningError::MissingCausalAnnotation {
                    record: classification.record.clone(),
                })?;
            let base_commit_id = record_policy_base_commit_id(
                &history,
                annotation,
                causal_plan.basis.merge_base.commit.commit_id,
            );
            let record_base_view =
                if base_commit_id == causal_plan.basis.merge_base.commit.commit_id {
                    &base_view
                } else {
                    let version_id = history
                        .commit_envelope(base_commit_id)
                        .ok_or(MergePlanningError::MissingMergeBaseEnvelope {
                            commit_id: base_commit_id,
                        })?
                        .commit
                        .version_id;
                    record_base_views
                        .entry(base_commit_id)
                        .or_insert_with(|| runtime.read_truth().read_version(execution, version_id))
                };
            let record_base_view_index =
                if base_commit_id == causal_plan.basis.merge_base.commit.commit_id {
                    &base_view_index
                } else {
                    record_base_view_indices
                        .entry(base_commit_id)
                        .or_insert_with(|| PolicyReadViewIndex::new(record_base_view))
                };
            let record_base_view_context =
                PolicyReadViewContext::new(record_base_view, record_base_view_index);
            let aspect_resolutions = resolve_aspects_for_record(
                runtime,
                record,
                classification,
                applied_policies.as_slice(),
                *causal_dispositions_by_record
                    .get(&classification.record)
                    .ok_or_else(|| MergePlanningError::MissingCausalAnnotation {
                        record: classification.record.clone(),
                    })?,
                &causal_plan.basis.source_head.branch_id,
                &causal_plan.basis.target_head.branch_id,
                base_commit_id,
                &source_view_context,
                &target_view_context,
                &record_base_view_context,
            )?;
            let ownership_surface = ownership_surface_for_policies(applied_policies.as_slice());
            let decision_boundary =
                aggregate_record_resolution(classification.class, aspect_resolutions.as_slice());
            Ok(MergePolicyResolutionRecord {
                record: classification.record.clone(),
                target_record: resolved_target_record_ref(
                    record,
                    classification.target_record.as_ref(),
                    &target_view_context,
                ),
                classification: classification.class,
                aspect_resolutions: Arc::from(aspect_resolutions),
                applied_policies: Arc::from(applied_policies),
                proof_boundary: MergePolicyProofBoundary {
                    ownership_surface,
                    decision_boundary,
                },
            })
        })
        .collect::<Result<Vec<_>, MergePlanningError>>()?;
    let policy_records: Arc<[MergePolicyResolutionRecord]> = Arc::from(policy_records);
    let policy_summary = summarize_policy_records(policy_records.clone());

    Ok(PolicyResolvedMergePlan {
        request: causal_plan.request,
        basis: causal_plan.basis,
        ancestry: causal_plan.ancestry,
        target_delta: causal_plan.target_delta,
        source_delta: causal_plan.source_delta,
        effective_identity_declarations: causal_plan.effective_identity_declarations,
        source_records: causal_plan.source_records,
        candidates: causal_plan.candidates,
        validated_schema_correspondences: causal_plan.validated_schema_correspondences,
        identity_summary: causal_plan.identity_summary,
        classifications: causal_plan.classifications,
        conflict_summary: causal_plan.conflict_summary,
        causal_annotations: causal_plan.causal_annotations,
        causal_summary: causal_plan.causal_summary,
        policy_records,
        policy_summary,
    })
}

fn resolve_aspects_for_record(
    runtime: &RelationalRuntime,
    record: &crate::merge::data::VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
    applied_policies: &[crate::merge::data::ResolvedAspectMergePolicy],
    causal_disposition: crate::merge::data::MergeRecordCausalDisposition,
    source_branch: &crate::history::data::BranchId,
    target_branch: &crate::history::data::BranchId,
    base_commit_id: crate::history::data::CommitId,
    source_view: &PolicyReadViewContext<'_>,
    target_view: &PolicyReadViewContext<'_>,
    base_view: &PolicyReadViewContext<'_>,
) -> Result<Vec<AspectPolicyResolutionRecord>, MergePlanningError> {
    let Some(lowered_plan) = lowered_plan_for_record(runtime, record) else {
        return Ok(Vec::new());
    };
    Ok(classification
        .aspect_evidence
        .iter()
        .map(|aspect| {
            let applied_policy = applied_policies
                .iter()
                .find(|policy| policy.aspect_key == aspect.aspect_key)
                .map(|policy| policy.policy.clone());
            let binding = lowered_plan.executable_bindings.iter().find(|binding| {
                super::binding_matches_aspect(runtime, binding, &aspect.aspect_key)
            });
            let initial_decision_boundary = decision_boundary_for_aspect(
                classification,
                aspect.comparison,
                applied_policy.as_ref(),
                causal_disposition,
            );
            let value_basis = policy_value_basis_for_aspect(
                runtime,
                record,
                classification,
                binding,
                applied_policy.as_ref(),
                source_branch,
                target_branch,
                base_commit_id,
                source_view,
                target_view,
                base_view,
            );
            let auto_resolution = resolve_aspect_value_strategy(
                record.record_kind.clone(),
                binding,
                value_basis.as_ref(),
                aspect.comparison,
                applied_policy.as_ref(),
                initial_decision_boundary,
                causal_disposition,
            );
            let (decision_boundary, resolved_value_strategy) = match auto_resolution {
                AutoResolutionStrategy::NotRequired => (initial_decision_boundary, None),
                AutoResolutionStrategy::Resolved(strategy) => {
                    (initial_decision_boundary, Some(strategy))
                }
                AutoResolutionStrategy::RequiresManual(class) => (
                    crate::merge::data::MergePolicyDecisionBoundary::RequiresManualResolution {
                        class,
                    },
                    None,
                ),
                AutoResolutionStrategy::Reject(class) => (
                    crate::merge::data::MergePolicyDecisionBoundary::Reject { class },
                    None,
                ),
            };
            AspectPolicyResolutionRecord {
                aspect_key: aspect.aspect_key.clone(),
                comparison: aspect.comparison,
                applied_policy,
                decision_boundary,
                resolved_value_strategy,
            }
        })
        .collect())
}

fn policy_value_basis_for_aspect(
    runtime: &RelationalRuntime,
    record: &crate::merge::data::VisibleMergeRecord,
    classification: &crate::merge::data::MergeConflictClassification,
    binding: Option<&crate::schema::data::LoweredAspectContractBinding>,
    applied_policy: Option<&crate::merge::data::AspectMergePolicyKind>,
    source_branch: &crate::history::data::BranchId,
    target_branch: &crate::history::data::BranchId,
    base_commit_id: crate::history::data::CommitId,
    source_view: &PolicyReadViewContext<'_>,
    target_view: &PolicyReadViewContext<'_>,
    base_view: &PolicyReadViewContext<'_>,
) -> Option<super::value_strategy::PolicyAspectValueBasis> {
    if !matches!(
        applied_policy,
        Some(crate::merge::data::AspectMergePolicyKind::MonotonicCounter)
    ) {
        return None;
    }
    let binding = match scalar_policy_aspect_binding(record.record_kind.clone(), binding) {
        Ok(binding) => binding,
        Err(ScalarPolicyBindingDenial::MissingBinding)
        | Err(ScalarPolicyBindingDenial::InvalidBuiltInPolicyValueShape) => return None,
    };
    let basis = resolve_policy_aspect_value_basis(
        runtime,
        record,
        classification,
        binding,
        source_branch,
        target_branch,
        base_commit_id,
        source_view,
        target_view,
        base_view,
    );
    let _basis_aspect_key = basis.binding().aspect_key();
    let _merge_basis = basis.merge_basis();
    let _merge_base_selection_basis = basis.merge_base_selection_basis();
    let _strategy_basis = basis.strategy_basis();
    let _policy_admission_receipt = basis.receipt().admission_receipt();
    let counters = basis.receipt().counters();
    runtime
        .performance_access()
        .count_merge_policy_value_lookup(
            counters.source_state_hit,
            counters.target_state_hit,
            counters.base_state_hit,
            counters.base_patch_authority_hit,
            counters.missing_ancestor_basis,
            counters.missing_visible_state,
            counters.invalid_shape,
        );
    Some(basis)
}

fn resolved_target_record_ref(
    record: &crate::merge::data::VisibleMergeRecord,
    target_record: Option<&crate::transactions::data::RecordRef>,
    target_view: &PolicyReadViewContext<'_>,
) -> Option<crate::transactions::data::RecordRef> {
    target_record.cloned().or_else(|| match record.record_ref {
        crate::transactions::data::RecordRef::Entity(entity_id) => target_view
            .entity_for_record(entity_id, record.target_lineage_id.or(record.lineage_id))
            .map(|entity| crate::transactions::data::RecordRef::Entity(entity.entity_id)),
        crate::transactions::data::RecordRef::Relation(relation_id) => target_view
            .relation_for_record(relation_id)
            .map(|relation| crate::transactions::data::RecordRef::Relation(relation.relation_id)),
    })
}

fn record_policy_base_commit_id(
    history: &crate::history::HistoryAccess,
    annotation: &crate::merge::data::MergeRecordCausalAnnotation,
    default_merge_base_commit_id: crate::history::data::CommitId,
) -> crate::history::data::CommitId {
    match (
        annotation
            .source_latest_touch
            .as_ref()
            .map(|dot| dot.commit_id),
        annotation
            .target_latest_touch
            .as_ref()
            .map(|dot| dot.commit_id),
    ) {
        (Some(source_commit_id), Some(target_commit_id)) => history
            .max_commit_id_common_ancestor(source_commit_id, target_commit_id)
            .unwrap_or(default_merge_base_commit_id),
        _ => default_merge_base_commit_id,
    }
}
