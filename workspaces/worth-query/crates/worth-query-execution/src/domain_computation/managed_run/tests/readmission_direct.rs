use super::yield_fixture::YieldProvider;
use super::*;

pub(super) fn yielded_direct(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
) -> (
    crate::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    yielded_direct_for_binding(execution, "managed-graph-binding")
}

pub(super) fn yielded_direct_for_binding(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    binding_identity: &str,
) -> (
    crate::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let (running, graph, bridge, runtime) = managed_graph_run_with_provider_and_runtime_binding(
        WorthQueryOperationGraphAccess::Observe,
        YieldProvider::installed(5),
        binding_identity,
    );
    let active = running
        .begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "direct-readmission",
            ),
        )
        .expect("yield provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("yield provider did not pause"),
    };
    let yielded = match paused.yield_run() {
        crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible run did not yield"),
    };
    (yielded, bridge, runtime)
}

pub(super) fn yielded_direct_with_plan_observation<T>(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    observe_plan: impl FnOnce(
        &worth_query_admission::facade::resource_admission::WorthQueryAdmittedExecutionResourcePlan,
    ) -> T,
) -> (
    crate::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
    T,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            YieldProvider::installed(5),
        ),
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
    let lower = causal_fixture::managed_admission_context();
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("managed graph run should admit through lower owners")
        .start();
    let bridge = lower.bridge;
    let active = running
        .begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "direct-readmission",
            ),
        )
        .expect("yield provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("yield provider did not pause"),
    };
    let yielded = match paused.yield_run() {
        crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible run did not yield"),
    };
    (yielded, bridge, runtime, plan_observation)
}

pub(in crate::domain_computation::managed_run) fn yielded_direct_with_provider(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
) -> (
    crate::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let (running, graph, bridge, runtime) = managed_graph_run_with_provider_and_runtime(
        WorthQueryOperationGraphAccess::Observe,
        provider,
    );
    let active = running
        .begin_graph_execution(
            execution,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "direct-readmission-provider-edge",
            ),
        )
        .expect("yield provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("yield provider did not pause"),
    };
    let yielded = match paused.yield_run() {
        crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("eligible run did not yield"),
    };
    (yielded, bridge, runtime)
}

#[test]
fn direct_readmission_mints_fresh_attempts_and_transfers_capacity() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct(execution);
        let logical = yielded.inspection().logical_run_identity().to_owned();
        let managed_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let provider_session = yielded.inspection().provider_session_identity().to_owned();
        let reservation_count = yielded.inspection().retained_capacity_reservation_count();
        let readmitted = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted,
            _ => panic!("same-runtime readmission should succeed"),
        };
        let active = readmitted.into_active();
        assert_eq!(active.logical_run_identity(), logical);
        assert_ne!(active.run_identity(), managed_attempt);
        assert_ne!(active.resource_attempt_identity(), resource_attempt);
        assert_ne!(active.provider_session_identity(), provider_session);
        assert_eq!(
            active.retained_capacity_reservation_count(),
            reservation_count
        );
        assert_eq!(reservation_count, 2);
        assert!(!active.provider_call_identity().is_empty());
        let readmitted_provider_session = active.provider_session_identity().to_owned();

        let completion = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("restored provider did not complete"),
        };
        let terminal = completion.into_running().completed().unwrap();
        assert_eq!(terminal.logical_run_identity(), logical);
        assert_eq!(terminal.provider_work().completed_work_units(), 4);
        assert_eq!(
            terminal.provider_work().provider_session_identity(),
            readmitted_provider_session
        );
        let cleanup = terminal.cleanup().expect("readmitted run should clean up");
        assert_eq!(
            cleanup.inspection().provider_session_identity(),
            readmitted_provider_session
        );
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .provider_session_identity(),
            readmitted_provider_session
        );
    });
}

#[test]
fn query_preflight_denial_returns_the_exact_yielded_capability_without_fresh_work() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) = yielded_direct(execution);
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let foreign_runtime = query_runtime();
        let denied = match yielded.readmit_same_runtime(active_request, &foreign_runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("foreign Query runtime should deny"),
        };
        assert_eq!(
            denied.kind(),
            crate::domain_computation::WorthQueryDirectReadmissionDenialKind::ForeignQueryRuntime
        );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.preflight_check_count(), 1);
        assert_eq!(counters.fresh_resource_attempt_count(), 0);
        assert_eq!(counters.bridge_readmission_attempt_count(), 0);
        assert!(evidence.bridge_counters().is_none());
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            resource_attempt
        );

        let active = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("returned yielded capability should remain readmittable"),
        };
        let completion = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("restored provider did not complete"),
        };
        assert!(completion
            .into_running()
            .completed()
            .unwrap()
            .cleanup()
            .is_ok());
    });
}

#[test]
fn provider_restore_denial_preserves_the_exact_checkpoint_and_capacity_package() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (yielded, bridge, runtime) =
            yielded_direct_with_provider(execution, YieldProvider::checkpoint_restore_failure(7));
        let checkpoint = yielded.inspection().checkpoint().identity().to_owned();
        let resource_attempt = yielded.inspection().yielded_attempt_identity().to_owned();
        let reservations = yielded.inspection().retained_capacity_reservation_count();
        let denied = match yielded.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => denied,
            _ => panic!("ordinary provider restore failure should deny"),
        };
        assert_eq!(
            denied.kind(),
            crate::domain_computation::WorthQueryDirectReadmissionDenialKind::ProviderRestoreDenied
        );
        let evidence = denied.readmission_evidence();
        let counters = evidence.query_counters();
        assert_eq!(counters.fresh_resource_attempt_count(), 1);
        assert_eq!(counters.bridge_readmission_attempt_count(), 1);
        assert_eq!(counters.provider_restore_attempt_count(), 1);
        assert_eq!(counters.committed_attempt_count(), 0);
        let bridge_counters = evidence
            .bridge_counters()
            .expect("provider denial must carry final Bridge abort evidence");
        assert_eq!(bridge_counters.abort_count(), 1);
        assert_eq!(bridge_counters.commit_count(), 0);
        let yielded = denied.into_yielded();
        assert_eq!(yielded.inspection().checkpoint().identity(), checkpoint);
        assert_eq!(
            yielded.inspection().yielded_attempt_identity(),
            resource_attempt
        );
        assert_eq!(
            yielded.inspection().retained_capacity_reservation_count(),
            reservations
        );
        let cleanup = complete_direct_yield_cleanup(yielded);
        assert_eq!(cleanup.checkpoint().unwrap().identity(), checkpoint);
    });
}
