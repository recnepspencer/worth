pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::entity_kind_id, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind};

pub(crate) fn vessel_call_signs(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> Vec<String> {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("fixture read request remains live")
        .expect("branch snapshot is readable");
    let aspect = worth_foundational::facade::AspectKey::new("call_sign").expect("call-sign aspect");
    view.entities()
        .iter()
        .filter(|record| record.kind.kind_id == entity_kind_id(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel))
        .filter_map(|record| {
            let state = record.authoritative_aspect_state.as_ref()?;
            let value = state.get(&aspect)?.view();
            match value {
                worth_foundational::facade::ContractValidatedAspectValueView::Scalar(
                    worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw(value)),
                ) => Some(value.clone()),
                worth_foundational::facade::ContractValidatedAspectValueView::Scalar(_)
                | worth_foundational::facade::ContractValidatedAspectValueView::Struct(_) => None,
            }
        })
        .collect()
}

pub(crate) fn live_record_count(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> usize {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    let snapshot = snapshot_for_supply_chain_identity(runtime, &identity);
    let view = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("fixture read request remains live")
        .expect("branch snapshot is readable");
    view.entities().len() + view.relations().len()
}

pub(crate) fn current_snapshot_version(runtime: &RelationalRuntime, branch: &BranchId) -> u64 {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    snapshot_for_supply_chain_identity(runtime, &identity)
        .version_id()
        .0
}