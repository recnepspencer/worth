//! A real Store home agrees with an independent memory runtime at every warmth.
pub(crate) use {crate::batches, crate::durable_run, crate::schema_transition, crate::workload};
pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::branch::RelationalBranchObservation, worth_relational::facade::history::BranchId, worth_relational::facade::identity::EntityId, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntime, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RecordRef};

pub(crate) fn observation(runtime: &RelationalRuntime) -> RelationalBranchObservation {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
        .observation()
}
pub(crate) fn render(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    selected: &RelationalBranchObservation,
    entities: &[EntityId],
) -> Vec<String> {
    let read = runtime
        .read_truth()
        .read_observation(execution, selected)
        .unwrap()
        .unwrap();
    entities
        .iter()
        .map(|id| format!("{:?}", read.get_entity(*id)))
        .collect()
}