//! One-way Native installation verifies both complete compiled inventories.
use super::native_meaning::{
    CompiledNativeSchemaMeaning, NativeSchemaInventory, PreparedNativeSchemaMeaning,
};
use super::*;

pub(super) fn install(
    relational_runtime: &mut RelationalRuntime,
    compiled: CompiledNativeSchemaMeaning,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    initialization: Option<
        worth_proof::FreshNativeInitialization<
            worth_relational::facade::durability::RelationalFreshInitialization,
        >,
    >,
) -> Result<PreparedNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
    let CompiledNativeSchemaMeaning {
        layout,
        additions,
        registrations,
        managed_sources,
        source_initializer,
    } = compiled;
    let mut inventory = NativeSchemaInventory::compile(&layout, &registrations);
    let public_source_programs = inventory.take_public_sources();
    let readmission_metadata_kinds = layout.home_readmission_metadata_kinds();
    let invariant_installation_receipt = match initialization {
        Some(initialization) => relational_runtime
            .prepare_fresh_home_schema_installation(request, initialization)
            .map_err(map_initial_schema_installation_denial)?
            .install_with_managed_sources_and_readmission_metadata(
                request,
                additions,
                registrations,
                public_source_programs,
                readmission_metadata_kinds,
            ),
        None => relational_runtime
            .prepare_initial_schema_installation()
            .map_err(map_initial_schema_installation_denial)?
            .install_with_managed_sources_and_readmission_metadata(
                additions,
                registrations,
                public_source_programs,
                readmission_metadata_kinds,
            ),
    }
    .map_err(map_initial_schema_installation_denial)?;
    inventory.require_receipt(&invariant_installation_receipt)?;
    let expected_invariant_inventory_digest = inventory.digest();
    Ok(PreparedNativeSchemaMeaning {
        layout,
        managed_sources: std::sync::Arc::new(managed_sources),
        source_initializer,
        receipt: invariant_installation_receipt,
        expected_invariant_inventory_digest,
    })
}
