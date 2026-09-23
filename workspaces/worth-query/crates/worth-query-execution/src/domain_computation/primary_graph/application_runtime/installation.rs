//! Construction phases for one published application runtime.
mod assembly;
mod input;
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
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledGraphParticipationAuthority,
};
pub(super) fn require_no_conditional_bindings<Schema>(
    runtime: &WorthQueryExecutionRuntime,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let count = runtime
        .installed_packages()
        .installed_conditional_node_count_for_schema(
            installed_schema.owner(),
            installed_schema.schema_name(),
        );
    if count == 0 {
        Ok(())
    } else {
        Err(WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::ConditionalBindingsRequired,
            format!(
                "{} declared conditional nodes require the conditional publication progression",
                count
            ),
        ))
    }
}
pub(super) fn publish_application_runtime_with_clock<Schema>(
    input: ApplicationRuntimePublication<Schema>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    let ApplicationRuntimePublication {
        bootstrap,
        runtime,
        authority,
        installed_schema,
        authorization_clock,
        fault_port,
        conditional_evaluation_budget,
    } = input;
    validate_application_schema(&runtime, &installed_schema)?;
    let mutation_projection = bootstrap.retain_invariant_projection_authority();
    let mutation_handlers = bootstrap
        .mutation_handlers
        .seal(&installed_schema, bootstrap.graph.binding_identity())?;
    let authorization = compile_authorization(&bootstrap, &installed_schema)?;
    let graph = publish_application_graph(
        bootstrap,
        runtime,
        authority,
        &installed_schema,
        fault_port,
        conditional_evaluation_budget,
    )?;
    let graph = seal_application_graph(graph)?;
    assemble_application_runtime(
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        Default::default(),
        mutation_handlers,
        mutation_projection,
        Default::default(),
        Default::default(),
    )
}
pub(in crate::domain_computation::primary_graph) fn publish_application_runtime_with_conditionals<
    Schema,
>(
    input: ApplicationRuntimePublication<Schema>,
    bindings: Vec<
        Box<dyn super::super::conditional_operation::WorthQueryPendingConditionalOperation<Schema>>,
    >,
    output_readiness: Vec<
        Box<dyn super::super::application_contribution::PendingOutputReadiness<Schema>>,
    >,
    producers: &super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    super::super::conditional_operation::WorthQueryConditionalRuntimeInstallationDenial,
>
where
    Schema: ApplicationSchema + 'static,
{
    let ApplicationRuntimePublication {
        bootstrap,
        runtime,
        authority,
        installed_schema,
        authorization_clock,
        fault_port,
        conditional_evaluation_budget,
    } = input;
    validate_application_schema(&runtime, &installed_schema)
        .map_err(super::super::conditional_operation::publication_denial)?;
    let mutation_projection = bootstrap.retain_invariant_projection_authority();
    let mutation_handlers = bootstrap
        .mutation_handlers
        .seal(&installed_schema, bootstrap.graph.binding_identity())
        .map_err(super::super::conditional_operation::publication_denial)?;
    let expected = runtime
        .installed_packages()
        .installed_conditional_node_count_for_schema(
            installed_schema.owner(),
            installed_schema.schema_name(),
        );
    super::super::conditional_operation::require_complete_binding_inventory(
        expected,
        &bindings,
        output_readiness.len(),
    )?;
    let authorization = compile_authorization(&bootstrap, &installed_schema)
        .map_err(super::super::conditional_operation::publication_denial)?;
    let mut graph = publish_application_graph(
        bootstrap,
        runtime,
        authority,
        &installed_schema,
        fault_port,
        conditional_evaluation_budget,
    )
    .map_err(super::super::conditional_operation::publication_denial)?;
    let authoritative_commit_cursor = graph.primary_provider.conditional_commit_sequence();
    let mut conditional_operations = super::super::conditional_operation::install_pending_bindings(
        bindings,
        graph.bridge.conditional_builder(),
        &graph.primary_graph_authority,
        authoritative_commit_cursor,
        graph.runtime.authority_identity().as_u64(),
        graph.runtime.installed_packages().runtime_ordinal(),
        graph.runtime.installed_packages().generation().ordinal(),
        graph.primary_graph_authority.provider_identity(),
        super::super::application_branch::PRIMARY_APPLICATION_BRANCH,
    )?;
    let output_producer_routes = producers
        .install_signal_routes(graph.bridge.conditional_builder())
        .map_err(super::super::conditional_operation::publication_denial)?;
    let output_readiness_routes =
        super::super::application_contribution::install_output_readiness_routes(
            output_readiness,
            graph.bridge.conditional_builder(),
            &graph.primary_graph_authority,
        )?;
    producers
        .validate_readiness_routes(&output_readiness_routes)
        .map_err(super::super::conditional_operation::publication_denial)?;
    graph
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .install_output_families(producers.output_family_bindings());
    let graph = seal_application_graph(graph)
        .map_err(super::super::conditional_operation::publication_denial)?;
    let mut application = assemble_application_runtime(
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        Default::default(),
        mutation_handlers,
        mutation_projection,
        output_producer_routes,
        output_readiness_routes,
    )
    .map_err(super::super::conditional_operation::publication_denial)?;
    conditional_operations.reconstruct_all(&application)?;
    conditional_operations.reconcile_all(&mut application.bridge.conditional_lifecycle())?;
    *application
        .conditional_operations
        .get_mut()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = conditional_operations;
    Ok(application)
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
}
fn validate_application_schema<Schema>(
    runtime: &WorthQueryExecutionRuntime,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
where
    Schema: ApplicationSchema,
{
    runtime
        .installed_packages()
        .validate_application_schema(installed_schema)
        .map_err(|denial| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                denial.subject(),
            )
        })
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
                denial.subject(),
            )
        })
}
fn publish_application_graph<Schema>(
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
    let bridge_layout = std::sync::Arc::clone(&bootstrap.graph.layout);
    let product_world_resources = bootstrap.product_world_resources.clone();
    let recovered_relational_authority = bootstrap.take_recovered_relational_authority();
    let publication = bootstrap.publish(&mut runtime, &authority)?;
    let graph = runtime
        .retain_primary_graph_integration_handle()
        .expect("publishing the primary graph installs its integration authority");
    let relational_source = graph.relational_bridge_source();
    graph
        .bind_current_truth_head(&super::super::application_branch::primary_relational_branch_id())
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
    let relational_branch_identity = graph.with_runtime(|runtime| {
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
        runtime.application_candidate_resource_profile(),
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
    })
}
