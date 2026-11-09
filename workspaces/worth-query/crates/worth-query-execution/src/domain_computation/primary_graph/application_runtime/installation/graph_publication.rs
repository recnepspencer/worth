//! Retire Native bootstrap publication before constructing Bridge service.
use super::*;
use crate::domain_computation::primary_graph::bootstrap::{
    BootstrapPublicationReservation, BootstrapPublicationStorage,
};
pub(super) struct PublishedNativeGraph {
    runtime: WorthQueryExecutionRuntime,
    resource_support: crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphResourceSupport,
    authority: WorthQueryExecutionInstallationAuthority,
    publication: super::super::WorthQueryPrimaryGraphPublication,
    bridge_layout: std::sync::Arc<super::super::super::WorthQueryPrimaryGraphLayout>,
    product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    recovered_relational_authority: Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    checkpoint_restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
}
#[inline(never)]
pub(super) fn publish_native<S: ApplicationSchema>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    mut bootstrap: WorthQueryPrimaryGraphBootstrap<S>,
    mut runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
) -> Result<
    BootstrapPublicationStorage<PublishedNativeGraph>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let bridge_layout = std::sync::Arc::clone(&bootstrap.graph.layout);
    let product_world_resources = bootstrap.product_world_resources.clone();
    let recovered_relational_authority = bootstrap.take_recovered_relational_authority();
    let checkpoint_restore_work = bootstrap.recovered_checkpoint_restore_work();
    let (publication, resource_support) =
        bootstrap.publish_with_resource_support(phase, &mut runtime, &authority)?;
    Ok(storage.fill(PublishedNativeGraph {
        runtime,
        resource_support,
        authority,
        publication,
        bridge_layout,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    }))
}
#[inline(never)]
pub(super) fn install_bridge<S: ApplicationSchema>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    seed: BootstrapPublicationStorage<PublishedNativeGraph>,
    installed_schema: &WorthQueryInstalledApplicationSchema<S>,
    fault_port: std::sync::Arc<
        dyn super::super::super::provider::fault_port::WorthQueryPrimaryGraphFaultPort,
    >,
    conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
) -> Result<
    BootstrapPublicationStorage<
        PublishedApplicationGraph<
            super::super::super::managed_bridge::WorthQueryApplicationBridgeInstallation,
        >,
    >,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())?;
    let PublishedNativeGraph {
        runtime,
        resource_support,
        authority,
        publication,
        bridge_layout,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    } = seed.consume();
    let request = phase.execution_request();
    let execution = &request;
    let graph = runtime
        .retain_primary_graph_integration_handle()
        .expect("publishing the primary graph installs its integration authority");
    let relational_source = graph.relational_bridge_source();
    graph
        .bind_current_truth_head(execution, &super::super::super::application_branch::primary_relational_branch_id())
        .map_err(|denial| {
            let kind = match denial {
                worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
                    WorthQueryPrimaryGraphInstallationDenialKind::RetentionCapacityExhausted
                }
                worth_relational::facade::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
                    WorthQueryPrimaryGraphInstallationDenialKind::RetentionIdentityExhausted
                }
                worth_relational::facade::branch::RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
                    WorthQueryPrimaryGraphInstallationDenialKind::SnapshotIdentityExhausted
                }
                _ => WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
            };
            WorthQueryPrimaryGraphInstallationDenial::new(kind, format!("{denial:?}"))
        })?;
    let relational_branch_identity = graph.with_runtime(execution, |runtime| {
        runtime
            .branch_identity(
                &super::super::super::application_branch::primary_relational_branch_id(),
            )
            .expect("the published primary application branch remains owner registered")
    });
    let bridge = super::super::super::managed_bridge::install_application_bridge(
        installed_schema,
        &bridge_layout,
        relational_source.clone(),
        conditional_evaluation_budget,
    )?;
    let truth_partition_role = graph.truth_partition_role().cloned();
    let maximum_concurrent_graph_work = runtime
        .application_query_resource_profile()
        .maximum_concurrent_graph_work();
    let (provider_anchor, primary_provider) = WorthQueryPrimaryGraphProvider::install(
        graph,
        fault_port,
        maximum_concurrent_graph_work,
        resource_support,
        installed_schema.installed_view().retain_operation_catalog(),
    );
    primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .install_binding_identities(
            installed_schema
                .installed_mutation_binding_inventory()
                .map(|binding| (TypeId::from(binding.binding_type()), binding.identity())),
        );
    let primary_graph_authority = super::super::graph_participation::install(
        &authority,
        truth_partition_role,
        provider_anchor,
    )?;
    Ok(storage.fill(PublishedApplicationGraph {
        runtime,
        publication,
        relational_branch_identity,
        bridge,
        primary_provider,
        primary_graph_authority,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    }))
}
