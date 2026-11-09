//! Construction phases for one published application runtime.

use crate::equality_type_id::EqualityTypeId as TypeId;
mod assembly;
pub(in crate::domain_computation::primary_graph) mod home_attachment;
mod input;
mod owned_contributions;
mod schema;
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
pub(in crate::domain_computation::primary_graph) use schema::validate_application_schema;
pub(super) use schema_admission::require_no_conditional_bindings;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledGraphParticipationAuthority,
};

mod clock_publication;
mod conditional_publication;
mod graph_publication;
mod graph_sealing;
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
            .with_authorization_cause(denial)
        })
}
fn publish_application_graph<Schema>(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
    runtime: WorthQueryExecutionRuntime,
    authority: WorthQueryExecutionInstallationAuthority,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
    fault_port: std::sync::Arc<
        dyn super::super::provider::fault_port::WorthQueryPrimaryGraphFaultPort,
    >,
    conditional_evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
) -> Result<
    crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationStorage<
        PublishedApplicationGraph<
            super::super::managed_bridge::WorthQueryApplicationBridgeInstallation,
        >,
    >,
    WorthQueryPrimaryGraphInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    let seed = graph_publication::publish_native(phase, bootstrap, runtime, authority)?;
    graph_publication::install_bridge(
        phase,
        seed,
        installed_schema,
        fault_port,
        conditional_evaluation_budget,
    )
}
#[inline(never)]
fn seal_application_graph(
    phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    graph: crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationStorage<
        PublishedApplicationGraph<
            super::super::managed_bridge::WorthQueryApplicationBridgeInstallation,
        >,
    >,
) -> Result<
    crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationStorage<
        PublishedApplicationGraph<
            super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
        >,
    >,
    WorthQueryPrimaryGraphInstallationDenial,
> {
    graph_sealing::seal(phase, graph)
}
