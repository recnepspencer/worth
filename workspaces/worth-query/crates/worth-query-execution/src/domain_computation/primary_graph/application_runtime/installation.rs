//! Construction phases for one published application runtime.

use crate::equality_type_id::EqualityTypeId as TypeId;
mod assembly;
pub(in crate::domain_computation::primary_graph) mod home_attachment;
mod input;
mod owned_contributions;
mod schema_admission;
use super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use assembly::assemble_application_runtime;
pub(in crate::domain_computation::primary_graph) use input::ApplicationRuntimePublication;
pub(super) use schema_admission::require_no_conditional_bindings;
use schema_admission::validate_application_schema;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledGraphParticipationAuthority,
};

mod clock_publication;
mod conditional_publication;
mod publication_preparation;
pub(super) fn publish_application_runtime_with_clock<Schema>(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    input: ApplicationRuntimePublication<Schema>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    let prepared = publication_preparation::prepare_clock(phase, input)?;
    let authorized = publication_preparation::authorize(phase, prepared)?;
    clock_publication::publish(phase, authorized)
}
pub(in crate::domain_computation::primary_graph) fn publish_application_runtime_with_conditionals<
    Schema,
>(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    input: ApplicationRuntimePublication<Schema>,
    bindings: Vec<
        Box<dyn super::super::conditional_operation::WorthQueryPendingConditionalOperation<Schema>>,
    >,
    output_readiness: Vec<
        Box<dyn super::super::application_contribution::PendingOutputReadiness<Schema>>,
    >,
    expected_conditional_bindings: usize,
    producers: &super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    super::super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Schema: ApplicationSchema + 'static,
{
    let prepared = publication_preparation::prepare_conditional(
        phase,
        input,
        expected_conditional_bindings,
        &bindings,
        output_readiness.len(),
    )?;
    let authorized = publication_preparation::authorize(phase, prepared)
        .map_err(super::super::conditional_operation::publication_denial)?;
    conditional_publication::publish(phase, authorized, bindings, output_readiness, producers)
}
pub(super) struct PublishedApplicationGraph<Bridge> {
    runtime: WorthQueryExecutionRuntime,
    publication: super::WorthQueryPrimaryGraphPublication,
    relational_branch_identity: worth_relational::facade::branch::RelationalBranchIdentity,
    bridge: Bridge,
    primary_provider: std::sync::Arc<WorthQueryPrimaryGraphProvider>,
    primary_graph_authority: WorthQueryInstalledGraphParticipationAuthority,
    product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    recovered_relational_authority:
        Option<worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority>,
    checkpoint_restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
}

fn compile_authorization<Schema>(
    bootstrap: &WorthQueryPrimaryGraphBootstrap<Schema>,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<WorthQueryInstalledAuthorizationRegistry, WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    WorthQueryInstalledAuthorizationRegistry::compile(installed_schema, &bootstrap.graph.layout)
        .map_err(|denial| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::AuthorizationPolicyRejected,
                denial.to_string(),
            )
        })
}
fn publish_application_graph<Schema>(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    mut bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
    mut runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    fault_port: std::sync::Arc<
        dyn super::super::provider::fault_port::WorthQueryPrimaryGraphFaultPort,
    >,
    conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
) -> Result<
    PublishedApplicationGraph<
        super::super::managed_bridge::WorthQueryApplicationBridgeInstallation,
    >,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    let execution_request = phase.execution_request();
    let execution = &execution_request;
    let bridge_layout = std::sync::Arc::clone(&bootstrap.graph.layout);
    let product_world_resources = bootstrap.product_world_resources.clone();
    let recovered_relational_authority = bootstrap.take_recovered_relational_authority();
    let checkpoint_restore_work = bootstrap.recovered_checkpoint_restore_work();
    let (publication, resource_support) =
        bootstrap.publish_with_resource_support(phase, &mut runtime, &authority)?;
    let graph = runtime
        .retain_primary_graph_integration_handle()
        .expect("publishing the primary graph installs its integration authority");
    let relational_source = graph.relational_bridge_source();
    graph
        .bind_current_truth_head(execution, &super::super::application_branch::primary_relational_branch_id())
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
            .branch_identity(&super::super::application_branch::primary_relational_branch_id())
            .expect("the published primary application branch remains owner registered")
    });
    let bridge = super::super::managed_bridge::install_application_bridge(
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
    let primary_graph_authority =
        super::graph_participation::install(&authority, truth_partition_role, provider_anchor)?;
    Ok(PublishedApplicationGraph {
        runtime,
        publication,
        relational_branch_identity,
        bridge,
        primary_provider,
        primary_graph_authority,
        product_world_resources,
        recovered_relational_authority,
        checkpoint_restore_work,
    })
}
fn seal_application_graph(
    graph: PublishedApplicationGraph<
        super::super::managed_bridge::WorthQueryApplicationBridgeInstallation,
    >,
) -> Result<
    PublishedApplicationGraph<super::super::managed_bridge::WorthQueryInstalledApplicationBridge>,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    Ok(PublishedApplicationGraph {
        runtime: graph.runtime,
        publication: graph.publication,
        relational_branch_identity: graph.relational_branch_identity,
        bridge: graph.bridge.seal().map_err(|denial| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
                format!("{denial:?}"),
            )
        })?,
        primary_provider: graph.primary_provider,
        primary_graph_authority: graph.primary_graph_authority,
        product_world_resources: graph.product_world_resources,
        recovered_relational_authority: graph.recovered_relational_authority,
        checkpoint_restore_work: graph.checkpoint_restore_work,
    })
}
