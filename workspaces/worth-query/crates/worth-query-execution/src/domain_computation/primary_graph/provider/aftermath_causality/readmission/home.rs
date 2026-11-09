//! One cold correction gate serves Home custody and the guarded transition lens.
use super::*;
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{RuntimeWorldInspectionPort, RuntimeWorldObservationPort};
pub(in crate::domain_computation::primary_graph) fn verify<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let owner = &application.runtime.product_runtime.owner;
    let graph = &application.runtime.primary_provider.graph;
    graph.with_runtime(execution, |native| {
        verify_source(
            execution,
            native,
            &graph.layout,
            &owner.inspection_port(),
            &owner.observation_port(),
        )
    })
}
pub(in crate::domain_computation::primary_graph) fn verify_source(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    inspection: &RuntimeWorldInspectionPort,
    observation: &RuntimeWorldObservationPort,
) -> Result<(), Denial> {
    let inventory = observation
        .observe_installed_product_branches(execution)
        .map_err(Denial::Inventory)?;
    let selected = inventory
        .branches()
        .first()
        .ok_or(Denial::ForeignWorldInventory)?;
    if inventory
        .branches()
        .iter()
        .any(|branch| branch.owner_identity() != selected.owner_identity())
    {
        return Err(Denial::ForeignWorldInventory);
    }
    let originals = inspection
        .observe_program_activation_history(execution)
        .map_err(Denial::WorldSource)?;
    let layout = layout.provider_aftermath_causality();
    let mut material = readmit_latest(
        execution,
        native,
        layout,
        &originals,
        selected.selected_commit(),
    )?;
    for branch in inventory.branches() {
        let first = readmit_branch(execution, native, layout, &originals, branch)?;
        material = material.or(first);
    }
    match material {
        Some(record) => Err(Denial::RetainedMaterialRequired { record }),
        None => Ok(()),
    }
}
