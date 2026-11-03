use std::sync::Arc;
use crate::domain_computation::managed_run::tests::ManagedGraph;
use worth_runtime_bridge::facade::RuntimeBridge;
use worth_query_execution::facade::domain_computation::WorthQueryDirectGraphStepOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use crate::domain_computation::provider_session::tests::admitted_plan_with_graph_support;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::operation_binding::authority::tests::direct_authority_with_graph;
use crate::domain_computation::managed_run::tests::managed_graph_run_with_provider_and_runtime;
use crate::domain_computation::managed_run::tests::managed_graph_run_with_provider_and_runtime_binding;

use crate::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
use crate::domain_computation::managed_run::tests::*;
pub fn yielded_direct(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    yielded_direct_for_binding(execution, "managed-graph-binding", resource_request)
}
pub fn yielded_direct_for_binding(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    binding_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let (running, graph, bridge, runtime) = managed_graph_run_with_provider_and_runtime_binding(
        WorthQueryOperationGraphAccess::Observe,
        YieldProvider::installed(5),
        binding_identity,
        resource_request,
    );
    let active = {
        let installed_fixture_phase = running.fixture_phase(execution);
        running.begin_graph_execution(
            &installed_fixture_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "direct-readmission",
            ),
        )
    }
    .expect("yield provider should begin");
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("yield provider did not pause"),
        }
    };
    let yielded = match paused.yield_run() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible run did not yield"),
    };
    (yielded, bridge, runtime)
}
pub fn yielded_direct_with_plan_observation<T>(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    observe_plan: impl FnOnce(
        &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan,
    ) -> T,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    T,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            _,
        >(YieldProvider::installed(5)),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        "managed-graph",
        provider_anchor.provider_identity(),
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
        "managed-graph-binding",
        8,
        graph.role(),
        provider_support,
    );
    let plan_observation = observe_plan(&plan);
    let operation = direct_authority_with_graph(
        &runtime,
        &plan,
        &graph,
        WorthQueryOperationGraphAccess::Observe,
    );
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("managed graph operation should start");
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("managed graph run should admit through lower owners")
        .start();
    let installed_running_phase = execution.phase_for_direct(&running);

    let bridge = lower.bridge;
    let active = running
        .begin_graph_execution(
            &installed_running_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "direct-readmission",
            ),
        )
        .expect("yield provider should begin");
    let paused = match active.advance(&installed_running_phase) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("yield provider did not pause"),
    };
    let yielded = match paused.yield_run() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible run did not yield"),
    };
    (yielded, bridge, runtime, plan_observation)
}
pub fn yielded_direct_with_provider(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: YieldProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let (running, graph, bridge, runtime) = managed_graph_run_with_provider_and_runtime(
        WorthQueryOperationGraphAccess::Observe,
        provider,
        resource_request,
    );
    let installed_running_phase = execution.phase_for_direct(&running);

    let active = running
        .begin_graph_execution(
            &installed_running_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "direct-readmission-provider-edge",
            ),
        )
        .expect("yield provider should begin");
    let paused = match active.advance(&installed_running_phase) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("yield provider did not pause"),
    };
    let yielded = match paused.yield_run() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible run did not yield"),
    };
    (yielded, bridge, runtime)
}
