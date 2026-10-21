//! A prepared read consumes its original exact schedule slot before execution.
use super::super::super::{
    batch::PlannedBatchItem, WorthQueryAdmittedApplicationQueryPlan,
    WorthQueryApplicationQueryBatchAdmission,
    WorthQueryApplicationQueryBatchResourceDenial as Resource,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
pub(in crate::domain_computation::primary_graph::application_query) fn admit_batch_read<
    S: ApplicationSchema,
    Q,
    P,
    R,
    A,
    I,
    C,
>(
    plan: &mut WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, C>,
    batch: &WorthQueryApplicationQueryBatchAdmission,
) -> Result<PlannedBatchItem, Resource> {
    match plan.planned_batch_item.take() {
        Some(item) if !item.belongs_to(batch) => Err(Resource::ForeignPlan),
        Some(item) => Ok(item),
        None => batch
            .take_planned_read(
                plan.runtime_authority.as_u64(),
                plan.query.binding_identity(),
                plan.query.identity(),
                plan.controls.maximum_result_count().get(),
            )?
            .ok_or(Resource::UnplannedRead),
    }
}
