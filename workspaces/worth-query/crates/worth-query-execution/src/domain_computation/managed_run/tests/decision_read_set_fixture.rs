use super::*;

pub(super) fn staged<'run>(
    execution: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: &'run mut WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run> {
    running
        .admit_provider_execution_plan(graph)
        .unwrap()
        .readmit(execution)
        .unwrap()
        .prepare()
        .unwrap()
        .bind_reads_and_effects()
}

pub(super) fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("decision-read fixture cleanup should complete");
}

pub(super) fn managed_decision_graph_run_with_provider<P>(
    provider: P,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + crate::domain_computation::WorthQueryProviderSessionLifecycle
        + crate::domain_computation::WorthQueryDecisionFactProvider,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_decision_capable::<ManagedGraph, P>(provider),
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
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("decision-capable managed run should admit")
        .start();
    (running, graph)
}

pub(super) fn managed_provisional_graph_run_with_provider<P>(
    provider: P,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + crate::domain_computation::WorthQueryProviderSessionLifecycle
        + crate::domain_computation::WorthQueryDecisionFactProvider
        + crate::domain_computation::WorthQueryProvisionalGraphProvider,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_provisional_capable::<ManagedGraph, P>(provider),
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
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("provisional-capable managed run should admit")
        .start();
    (running, graph)
}

pub(super) fn managed_invariant_graph_run_with_provider<P>(
    provider: P,
    families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    invariants: Vec<
        worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
    >,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
)
where
    P: WorthQueryGraphParticipationProvider<ManagedGraph>
        + crate::domain_computation::WorthQueryProviderSessionLifecycle
        + crate::domain_computation::WorthQueryDecisionFactProvider
        + crate::domain_computation::WorthQueryProvisionalGraphProvider
        + crate::domain_computation::WorthQueryInvariantExecutionProvider,
{
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install_invariant_capable::<ManagedGraph, P>(provider),
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
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("invariant-capable managed run should admit")
        .start();
    (running, graph)
}
