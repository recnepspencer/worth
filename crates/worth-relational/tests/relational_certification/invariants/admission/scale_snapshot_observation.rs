use worth_foundational::facade::{AspectKey, AspectValue, InternedString};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

use super::super::world::supply_chain::{
    entity_kind_id, snapshot_for_supply_chain_identity, EntityKind,
};

pub(super) fn vessel_call_signs(
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
    let aspect = AspectKey::new("call_sign").expect("call-sign aspect");
    view.entities()
        .iter()
        .filter(|record| record.kind.kind_id == entity_kind_id(EntityKind::Vessel))
        .filter_map(|record| {
            let state = record.authoritative_aspect_state.as_ref()?;
            let value = state.get(&aspect)?.view();
            match value {
                worth_foundational::facade::ContractValidatedAspectValueView::Scalar(
                    AspectValue::String(InternedString::Raw(value)),
                ) => Some(value.clone()),
                worth_foundational::facade::ContractValidatedAspectValueView::Scalar(_)
                | worth_foundational::facade::ContractValidatedAspectValueView::Struct(_) => None,
            }
        })
        .collect()
}

pub(super) fn live_record_count(
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

pub(super) fn current_snapshot_version(runtime: &RelationalRuntime, branch: &BranchId) -> u64 {
    let identity = runtime
        .branch_identity(branch)
        .expect("branch identity is owner-issued");
    snapshot_for_supply_chain_identity(runtime, &identity)
        .version_id()
        .0
}
