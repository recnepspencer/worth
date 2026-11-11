//! Installed application mappings before Store attaches its sole Native source.
use super::*;
use worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder;

pub(in crate::domain_computation::primary_graph) fn prepare_home_program_mappings<
    Schema: ApplicationSchema,
>(
    incumbent: &RuntimeBridge,
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
) -> Result<RuntimeBridge, WorthQueryPrimaryGraphInstallationDenial> {
    let (routing, aspects) = application_mappings(schema, layout)?
        .map(|mapping| (mapping.routing, mapping.aspect))
        .unzip();
    incumbent
        .with_uninstalled_home_program_mappings(routing, aspects)
        .map_err(|cause| bridge_denial(format!("Home successor mappings refused: {cause:?}")))
}

pub(in crate::domain_computation::primary_graph) fn prepare_home_bridge_definition<
    Schema: ApplicationSchema,
>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    layout: &super::super::schema_layout::WorthQueryPrimaryGraphLayout,
) -> Result<UnattachedRelationalBridgeBuilder, WorthQueryPrimaryGraphInstallationDenial> {
    let mut mappings = application_mappings(schema, layout)?;
    let first = mappings.next().ok_or_else(|| {
        bridge_denial("installed application schema has no bridge-readable fields")
    })?;
    let builder = RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::operational())
        .with_signal_sink(WorthQueryApplicationGranularOnlySink)
        .register_source(SourceDeclaration::new(
            SourceDeclarationIdentity::from_stable_name("primary-application-source"),
            BridgeTruthViewSelector::branch_head(
                super::super::application_branch::primary_truth_branch_identity(),
            ),
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ]),
        ))
        .register_aspect_mapping(first.aspect)
        .register_mapping(first.routing);
    Ok(mappings.fold(builder, |builder, mapping| {
        builder
            .register_aspect_mapping(mapping.aspect)
            .register_mapping(mapping.routing)
    }))
}
