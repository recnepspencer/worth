//! Construction phases for one published application runtime.

mod input;
pub(in crate::domain_computation::primary_graph) use input::ApplicationRuntimePublication;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledGraphParticipationAuthority,
};

use super::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
    WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::authorization::{
    WorthQueryInstalledAuthorizationRegistry, WorthQueryRuntimeClock,
};
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::primary_graph::authentication_clock::WorthQueryAuthenticationClock;

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
    )
}

pub(in crate::domain_computation::primary_graph) fn publish_application_runtime_with_conditionals<
    Schema,
>(
    input: ApplicationRuntimePublication<Schema>,
    bindings: Vec<
        Box<dyn super::super::conditional_operation::WorthQueryPendingConditionalOperation<Schema>>,
    >,
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
    let expected = runtime
        .installed_packages()
        .installed_conditional_node_count_for_schema(
            installed_schema.owner(),
            installed_schema.schema_name(),
        );
    super::super::conditional_operation::require_complete_binding_inventory(expected, &bindings)?;
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
    let graph = seal_application_graph(graph)
        .map_err(super::super::conditional_operation::publication_denial)?;
    let mut application = assemble_application_runtime(
        graph,
        installed_schema,
        authorization,
        authorization_clock,
        Default::default(),
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
    bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
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
    let (provider_anchor, primary_provider) =
        WorthQueryPrimaryGraphProvider::install(graph, fault_port);
    let primary_graph_authority =
        install_graph_participation_authority(&authority, truth_partition_role, provider_anchor)?;
    Ok(PublishedApplicationGraph {
        runtime,
        publication,
        relational_branch_identity,
        bridge,
        primary_provider,
        primary_graph_authority,
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
    })
}

fn install_graph_participation_authority(
    authority: &WorthQueryExecutionInstallationAuthority,
    truth_partition_role: Option<worth_foundational::facade::TruthPartitionRole>,
    provider_anchor: std::sync::Arc<
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor,
    >,
) -> Result<WorthQueryInstalledGraphParticipationAuthority, WorthQueryPrimaryGraphInstallationDenial>
{
    WorthQueryInstalledGraphParticipationAuthority::install_with_truth_partition(
        authority.installation_runtime(),
        "primary",
        provider_anchor.provider_identity(),
        true,
        Some("primary"),
        truth_partition_role,
        provider_anchor,
    )
    .map_err(|detail| {
        WorthQueryPrimaryGraphInstallationDenial::new(
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
            detail,
        )
    })
}

fn assemble_application_runtime<Schema>(
    graph: PublishedApplicationGraph<
        super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
    >,
    installed_schema: WorthQueryInstalledApplicationSchema<Schema>,
    authorization: WorthQueryInstalledAuthorizationRegistry,
    authorization_clock: WorthQueryRuntimeClock,
    conditional_operations:
        super::super::conditional_operation::WorthQueryConditionalOperationRegistry<Schema>,
) -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let product_runtime = crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime::install(
        graph.primary_provider.graph.prepare_product_source(&graph.relational_branch_identity)
            .map_err(|denial| WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected,
                format!("Product source admission: {denial:?}"),
            ))?,
        &mut graph.bridge.conditional_lifecycle(),
    ).map_err(|denial| WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        denial.detail(),
    ))?;
    let runtime_authority = graph.runtime.authority_identity();
    let schema_binding = installed_schema.binding_identity();
    let application_readiness_schema_token = format!(
        "{}:{}:{}",
        schema_binding.generation(),
        schema_binding.package_identity().render_hex(),
        schema_binding.schema_identity().render_hex(),
    );
    let granular_invalidation = super::super::WorthQueryGranularInvalidationInstallation::new(
        schema_binding.clone(),
        graph.primary_provider.graph.clone(),
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductSharedRoot::new(
            product_runtime.clone(),
            graph.bridge.conditional_operations(),
        ),
    );
    // One clock, shared. The registry hands it back to any handle that needs to
    // re-check its own deadline, which is why no recovery transition takes a
    // clock argument (R8.31).
    let authorization_clock = std::sync::Arc::new(authorization_clock);
    let recovery_handles = std::sync::Arc::new(
        crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry::for_runtime(
            runtime_authority,
            std::sync::Arc::clone(&authorization_clock),
        ),
    );
    Ok(WorthQueryPrimaryGraphApplicationRuntime {
        runtime: graph.runtime,
        installed_schema,
        application_readiness_schema_token,
        publication: graph.publication,
        authorization,
        authorization_clock,
        authentication_clock: WorthQueryAuthenticationClock::system(),
        relational_branch_identity: graph.relational_branch_identity,
        bridge: graph.bridge,
        product_runtime,
        granular_invalidation,
        conditional_operations: std::sync::Mutex::new(conditional_operations),
        primary_provider: graph.primary_provider,
        primary_graph_authority: graph.primary_graph_authority,
        result_buffers: Default::default(),
        basis_leases: Default::default(),
        next_preview_session: std::sync::atomic::AtomicU64::new(1),
        next_external_dispatch_attempt: std::sync::atomic::AtomicU64::new(1),
        external_effect_transport: std::sync::OnceLock::new(),
        recovery_handles,
    })
}
