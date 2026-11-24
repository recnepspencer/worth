use crate::merge::aspect_components::{binding_component_from_visible_record, VisibleRecordSide};
use crate::merge::aspect_plan_lookup::lowered_plan_for_record;
use crate::merge::data::{AspectComparisonState, AspectConflictEvidence, VisibleMergeRecord};
use crate::schema::data::AspectContractPlanCatalog;
use crate::schema::data::LoweredAspectContractBinding;

pub(super) fn aspect_conflict_evidence(
    aspect_plans: &AspectContractPlanCatalog,
    source_record: &VisibleMergeRecord,
    target_record: Option<&VisibleMergeRecord>,
) -> Vec<AspectConflictEvidence> {
    let Some(plan) = lowered_plan_for_record(aspect_plans, source_record) else {
        return Vec::new();
    };
    plan.executable_bindings
        .iter()
        .map(|binding| AspectConflictEvidence {
            aspect_key: binding.aspect_key().clone(),
            comparison: compare_binding(source_record, target_record, binding),
        })
        .collect()
}

fn compare_binding(
    source_record: &VisibleMergeRecord,
    target_record: Option<&VisibleMergeRecord>,
    binding: &LoweredAspectContractBinding,
) -> AspectComparisonState {
    let source =
        binding_component_from_visible_record(source_record, binding, VisibleRecordSide::Source);
    let target = target_record.and_then(|record| {
        binding_component_from_visible_record(record, binding, VisibleRecordSide::Target)
    });
    match (source, target) {
        (Some(source), Some(target)) if source == target => AspectComparisonState::Equal,
        (Some(_), Some(_)) => AspectComparisonState::Divergent,
        (Some(_), None) => AspectComparisonState::SourceOnly,
        (None, Some(_)) => AspectComparisonState::TargetOnly,
        (None, None) => AspectComparisonState::Unavailable,
    }
}
