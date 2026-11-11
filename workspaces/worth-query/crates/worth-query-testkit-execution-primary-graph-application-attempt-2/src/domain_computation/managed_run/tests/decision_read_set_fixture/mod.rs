use crate::domain_computation::managed_run::tests::*;

pub fn staged<'run>(
    execution: &'run worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    running: &'run mut WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> worth_query_execution::facade::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>
{
    {
        let installed_fixture_phase = running.fixture_phase_for_test(execution);
        running
            .admit_provider_execution_plan(graph)
            .unwrap()
            .admit_session(&installed_fixture_phase)
            .and_then(|(grant, receiver)| grant.prepare(receiver, &installed_fixture_phase))
    }
    .unwrap()
    .bind_reads_and_effects()
}

pub fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence_for_test(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("decision-read fixture cleanup should complete");
}

pub fn managed_decision_graph_run_with_provider<P>(
    provider: P,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::domain_computation::WorthQueryDecisionFactProvider,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install_decision_capable::<ManagedGraph, P>(provider),
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
    .expect("decision-capable graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("decision-capable graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        "managed-graph-binding",
        8,
        graph.role(),
        provider_support,
    );
    let operation = direct_authority_with_graph_and_decision_facts(
        &runtime,
        &plan,
        &graph,
        WorthQueryOperationGraphAccess::Observe,
        families,
    );
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("decision-capable graph attempt should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("decision-capable managed run should admit")
        .start();
    (running, graph)
}

pub fn managed_provisional_graph_run_with_provider<P>(
    provider: P,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::domain_computation::WorthQueryDecisionFactProvider
        + worth_query_execution::facade::domain_computation::WorthQueryProvisionalGraphProvider,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install_provisional_capable::<ManagedGraph, P>(provider),
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
    .expect("provisional-capable graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("provisional-capable graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        "managed-graph-binding",
        8,
        graph.role(),
        provider_support,
    );
    let operation =
        direct_authority_with_graph_effect_and_decision_facts(&runtime, &plan, &graph, families);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("provisional-capable graph attempt should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("provisional-capable managed run should admit")
        .start();
    (running, graph)
}

pub fn managed_invariant_graph_run_with_provider<P>(
    provider: P,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    invariants: Vec<
        worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
    >,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle
        + worth_query_execution::facade::domain_computation::WorthQueryDecisionFactProvider
        + worth_query_execution::facade::domain_computation::WorthQueryProvisionalGraphProvider
        + worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionProvider,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install_invariant_capable::<ManagedGraph, P>(provider),
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
    .expect("invariant-capable graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("invariant-capable graph runtime should install")
        .into_parts()
        .0;
    let plan = admitted_plan_with_graph_support(
        "managed-graph-binding",
        8,
        graph.role(),
        provider_support,
    );
    let operation = direct_authority_with_graph_effect_decision_facts_and_invariants(
        &runtime, &plan, &graph, families, invariants,
    );
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("invariant-capable graph attempt should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("invariant-capable managed run should admit")
        .start();
    (running, graph)
}
