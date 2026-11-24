use crate::merge::data::{VisibleMergeRecord, VisibleMergeRecordKind};
use worth_relational_contracts::internal_port::schema::data::aspect_contracts::lowered_plan::{
    AspectContractPlanCatalog, LoweredAspectContractPlan,
};

pub(crate) fn lowered_plan_for_record<'a>(
    aspect_plans: &'a AspectContractPlanCatalog,
    record: &VisibleMergeRecord,
) -> Option<&'a LoweredAspectContractPlan> {
    let kind_id = record.source_kind_id.or(record.kind_id)?;
    match record.record_kind {
        VisibleMergeRecordKind::Entity => aspect_plans.entity_plans.get(&kind_id),
        VisibleMergeRecordKind::Relation => aspect_plans.relation_plans.get(&kind_id),
    }
}
