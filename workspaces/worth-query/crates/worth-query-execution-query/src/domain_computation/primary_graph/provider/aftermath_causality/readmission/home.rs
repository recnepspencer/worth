//! One cold correction gate serves Home custody and the guarded transition lens.
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_query_execution_attempt::facade::domain_computation::primary_graph::provider::aftermath_causality::readmission::{readmit_branch, readmit_latest_with_unpublished, QueryHomeCorrectionLineageDenial as Denial};
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{RuntimeWorldInspectionPort, RuntimeWorldObservationPort};
pub fn verify<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    metadata_authority: Option<
        &worth_relational::facade::durability::HomeReadmissionMetadataAuthority,
    >,
) -> Result<(), Denial> {
    verify_with_unpublished(
        application,
        execution,
        &std::collections::BTreeMap::new(),
        &std::collections::BTreeSet::new(),
        metadata_authority,
    )
}
pub fn verify_with_unpublished<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &ExecutionRequest<'_, '_>,
    continuable: &std::collections::BTreeMap<CommitId, DurableCompositePublicationAttemptIdentity>,
    all_unpublished: &std::collections::BTreeSet<CommitId>,
    metadata_authority: Option<
        &worth_relational::facade::durability::HomeReadmissionMetadataAuthority,
    >,
) -> Result<(), Denial> {
    let owner = &application.runtime.product_runtime.owner_ref();
    let graph = &application.runtime.primary_provider.graph_ref();
    graph.with_runtime(execution, |native| {
        verify_with_material(
            execution,
            native,
            graph.layout_ref(),
            &owner.inspection_port(),
            &owner.observation_port(),
            Some(&application.runtime.primary_provider),
            (continuable, all_unpublished),
            metadata_authority,
        )
    })
}
pub fn verify_source(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    inspection: &RuntimeWorldInspectionPort,
    observation: &RuntimeWorldObservationPort,
) -> Result<(), Denial> {
    let empty_map = std::collections::BTreeMap::new();
    let empty_set = std::collections::BTreeSet::new();
    verify_with_material(
        execution,
        native,
        layout,
        inspection,
        observation,
        None,
        (&empty_map, &empty_set),
        None,
    )
}
fn verify_with_material(
    execution: &ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    inspection: &RuntimeWorldInspectionPort,
    observation: &RuntimeWorldObservationPort,
    material_owner: Option<
        &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    >,
    unpublished: (
        &std::collections::BTreeMap<CommitId, DurableCompositePublicationAttemptIdentity>,
        &std::collections::BTreeSet<CommitId>,
    ),
    metadata_authority: Option<
        &worth_relational::facade::durability::HomeReadmissionMetadataAuthority,
    >,
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
    let mut material = readmit_latest_with_unpublished(
        execution,
        native,
        layout,
        &originals,
        selected.selected_commit(),
        material_owner,
        unpublished,
        metadata_authority,
    )?;
    for branch in inventory.branches() {
        let first = readmit_branch(
            execution,
            native,
            layout,
            &originals,
            branch,
            material_owner,
            metadata_authority,
        )?;
        material = material.or(first);
    }
    match material {
        Some(record) if material_owner.is_none() => {
            Err(Denial::RetainedMaterialRequired { record })
        }
        Some(_) => Ok(()),
        None => Ok(()),
    }
}
