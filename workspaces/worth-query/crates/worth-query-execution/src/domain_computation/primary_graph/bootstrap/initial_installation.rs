//! One-way Native installation verifies both complete compiled inventories.
use super::*;
use worth_relational::facade::{
    runtime::{CustomInvariantRegistration, RelationalInitialSchemaInstallationReceipt},
    schema::RelationalSchemaRegistry,
};

pub(super) fn install(
    relational_runtime: &mut RelationalRuntime,
    additions: RelationalSchemaRegistry,
    registrations: Vec<CustomInvariantRegistration>,
    layout: &WorthQueryPrimaryGraphLayout,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    initialization: Option<
        worth_proof::FreshNativeInitialization<
            worth_relational::facade::durability::RelationalFreshInitialization,
        >,
    >,
) -> Result<
    (RelationalInitialSchemaInstallationReceipt, [u8; 32]),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let expected_invariant_inventory_digest =
        worth_relational::facade::runtime::custom_invariant_inventory_digest(&registrations);
    let expected_source_programs = layout
        .required_managed_sources()
        .map(|source| source.program.clone())
        .collect::<Vec<_>>();
    let public_source_programs = layout
        .required_managed_sources()
        .filter(|source| source.definition.maintenance().native_cold().is_some())
        .map(|source| source.program.clone())
        .collect();
    let invariant_installation_receipt = match initialization {
        Some(initialization) => relational_runtime
            .prepare_fresh_home_schema_installation(request, initialization)
            .map_err(map_initial_schema_installation_denial)?
            .install_with_managed_sources(
                request,
                additions,
                registrations,
                public_source_programs,
            ),
        None => relational_runtime
            .prepare_initial_schema_installation()
            .map_err(map_initial_schema_installation_denial)?
            .install_with_managed_sources(additions, registrations, public_source_programs),
    }
    .map_err(map_initial_schema_installation_denial)?;
    if invariant_installation_receipt.custom_invariant_inventory_digest()
        != &expected_invariant_inventory_digest
    {
        return Err(primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch,
            "Relational installed an invariant inventory outside the application catalog",
        ));
    }
    if !invariant_installation_receipt.has_exact_managed_index_sources(&expected_source_programs) {
        return Err(primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::ManagedSourceInstallationReceiptMismatch,
            "Relational installed a Source roster outside the compiled application catalog",
        ));
    }
    Ok((
        invariant_installation_receipt,
        expected_invariant_inventory_digest,
    ))
}
