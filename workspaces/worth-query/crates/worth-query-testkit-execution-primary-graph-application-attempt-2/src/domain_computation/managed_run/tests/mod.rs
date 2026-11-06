use crate::domain_computation::managed_run::tests::graph_material::graph_material;
pub use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::direct_yield_cleanup_fixture::complete_direct_yield_cleanup;
use std::sync::Arc;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::InternedString;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use worth_runtime_bridge::facade::RuntimeBridge;
use worth_query_execution::facade::domain_computation::WorthQueryManagedRunTerminalKind;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority_with_graph;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority_with_graph_and_decision_facts;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority_with_graph_effect;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority_with_graph_effect_and_decision_facts;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority_with_graph_effect_decision_facts_and_invariants;
use crate::domain_computation::provider_session::tests::admitted_plan_with_graph_support;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderResource;
use worth_query_execution::facade::domain_computation::WorthQueryCooperativeGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphParticipationProvider;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCall;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecutionStart;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderFailure;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderRestoreMemory;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderRetainedMemory;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStep;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDisposition;
use worth_query_execution::facade::domain_computation::WorthQueryGraphReadMaterial;
use worth_query_execution::facade::domain_computation::WorthQueryGraphReadRow;
use worth_query_execution::facade::domain_computation::WorthQueryRunningDirectRun;
pub fn admit_provider_execution<E: WorthQueryGraphProviderExecution>(
    start: &mut WorthQueryGraphProviderExecutionStart,
    execution: E,
) -> Result<WorthQueryCooperativeGraphProviderExecution<E>, WorthQueryGraphProviderFailure> {
    start
        .admit_cooperative_execution(|| execution)
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))
}
pub fn admit_restored_provider_execution(
    memory: &mut WorthQueryGraphProviderRestoreMemory,
    execution: Box<dyn WorthQueryGraphProviderExecution>,
) -> Result<
    WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
    WorthQueryGraphProviderFailure,
> {
    memory
        .admit_cooperative_execution(|| execution)
        .map_err(|denial| WorthQueryGraphProviderFailure::new(denial.detail()))
}
pub fn query_runtime() -> WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("managed-run runtime should install")
        .into_parts()
        .0
}
#[derive(Clone, Copy)]
pub struct ManagedGraph;
pub fn managed_graph_run_with_provider<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let (running, graph, _bridge) =
        managed_graph_run_with_provider_and_bridge(access, provider, resource_request);
    (running, graph)
}
pub fn managed_graph_run_with_provider_and_bridge<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let (running, graph, bridge, _runtime) = managed_graph_run_with_provider_and_admitted_support(
        provider,
        ManagedGraphRunConfiguration {
            access,
            touch: false,
            binding_identity: "managed-graph-binding",
        },
        |support| support.clone(),
        resource_request,
    );
    (running, graph, bridge)
}
pub fn managed_graph_run_with_provider_and_runtime<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    managed_graph_run_with_provider_and_runtime_binding(
        access,
        provider,
        "managed-graph-binding",
        resource_request,
    )
}
pub fn managed_graph_run_with_provider_and_runtime_binding<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    binding_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    managed_graph_run_with_provider_and_admitted_support(
        provider,
        ManagedGraphRunConfiguration {
            access,
            touch: false,
            binding_identity,
        },
        |support| support.clone(),
        resource_request,
    )
}
pub fn managed_graph_effect_run_with_provider<P>(
    provider: P,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let (running, graph, _, _runtime) = managed_graph_run_with_provider_and_admitted_support(
        provider,
        ManagedGraphRunConfiguration {
            access: WorthQueryOperationGraphAccess::Observe,
            touch: true,
            binding_identity: "managed-graph-binding",
        },
        |support| support.clone(),
        resource_request,
    );
    (running, graph)
}
pub struct ManagedGraphRunConfiguration<'a> {
    pub access: WorthQueryOperationGraphAccess,
    pub touch: bool,
    pub binding_identity: &'a str,
}
pub fn managed_graph_run_with_provider_and_admitted_support<P>(
    provider: P,
    configuration: ManagedGraphRunConfiguration<'_>,
    admitted_support: impl FnOnce(
        &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            P,
        >(provider),
    );
    let provider_identity = provider_anchor.provider_identity();
    let provider_support = admitted_support(provider_anchor.resource_support());
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        provider_identity,
        false,
        Option::<String>::None,
        provider_anchor,
    )
    .expect("managed graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("managed graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        configuration.binding_identity,
        8,
        graph.role(),
        provider_support,
    );
    let operation = if configuration.touch {
        direct_authority_with_graph_effect(&runtime, &plan, &graph)
    } else {
        direct_authority_with_graph(&runtime, &plan, &graph, configuration.access)
    };
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("managed graph operation should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("managed graph run should admit through lower owners")
        .start();
    (running, graph, lower.bridge, runtime)
}
pub fn managed_session_graph_run_with_provider<P>(
    access: WorthQueryOperationGraphAccess,
    provider: P,
    touch: bool,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install_session_capable::<ManagedGraph, P>(provider),
    );
    let provider_identity = provider_anchor.provider_identity();
    let provider_support = provider_anchor.resource_support().clone();
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        provider_identity,
        false,
        Option::<String>::None,
        provider_anchor,
    )
    .expect("session-capable graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("session-capable graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        "managed-graph-binding",
        8,
        graph.role(),
        provider_support,
    );
    let operation = if touch {
        direct_authority_with_graph_effect(&runtime, &plan, &graph)
    } else {
        direct_authority_with_graph(&runtime, &plan, &graph, access)
    };
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("session-capable graph attempt should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("session-capable managed run should admit")
        .start();
    (running, graph)
}

#[path = "combined_provider_capability/mod.rs"]
pub mod combined_provider_capability;
#[path = "cost_bound/mod.rs"]
pub mod cost_bound;
#[path = "decision_read_set/mod.rs"]
pub mod decision_read_set;
#[path = "decision_read_set_fixture/mod.rs"]
pub mod decision_read_set_fixture;
#[path = "graph_material/mod.rs"]
pub mod graph_material;
#[path = "invariant_execution/mod.rs"]
pub mod invariant_execution;
#[path = "invariant_execution_fixture/mod.rs"]
pub mod invariant_execution_fixture;
#[path = "provider_session_protocol/mod.rs"]
pub mod provider_session_protocol;
#[path = "provider_work/mod.rs"]
pub mod provider_work;
#[path = "provisional_attempt/mod.rs"]
pub mod provisional_attempt;
#[path = "provisional_attempt_fixture/mod.rs"]
pub mod provisional_attempt_fixture;
#[path = "readmission_custody/mod.rs"]
pub mod readmission_custody;
#[path = "readmission_direct/mod.rs"]
pub mod readmission_direct;
#[path = "readmission_workflow/mod.rs"]
pub mod readmission_workflow;
#[path = "workflow_provider_steps/mod.rs"]
pub mod workflow_provider_steps;
#[path = "yield_checkpoint_fixture/mod.rs"]
pub mod yield_checkpoint_fixture;
#[path = "yield_fixture/mod.rs"]
pub mod yield_fixture;
#[path = "yield_provider_configuration/mod.rs"]
pub mod yield_provider_configuration;
