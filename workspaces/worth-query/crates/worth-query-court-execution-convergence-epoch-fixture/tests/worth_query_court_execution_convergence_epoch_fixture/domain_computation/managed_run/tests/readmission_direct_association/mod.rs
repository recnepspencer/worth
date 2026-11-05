pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
pub use crate::domain_computation::managed_run::tests::*;

pub fn shared_yielded_peers(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    shared_yielded_peers_with_provider(execution, YieldProvider::installed(5), resource_request)
}

pub fn shared_yielded_peers_with_provider(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: YieldProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        worth_query_execution::facade::integration::WorthQueryGraphProviderAnchor::install::<
            ManagedGraph,
            YieldProvider,
        >(provider),
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
    .expect("shared graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("shared execution runtime should install")
        .into_parts()
        .0;
    let lower = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request);
    let first = yield_peer(
        execution,
        &runtime,
        &graph,
        &lower,
        provider_support.clone(),
        "direct-peer",
        resource_request,
    );
    let second = yield_peer(
        execution,
        &runtime,
        &graph,
        &lower,
        provider_support,
        "direct-peer",
        resource_request,
    );
    (first, second, lower.bridge, runtime)
}

#[test]
fn cleanup_pending_peers_keep_exact_yielded_association_through_rightful_retry() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (first, second, bridge, runtime) = shared_yielded_peers_with_provider(
            execution,
            YieldProvider::checkpoint_restore_panic(5),
            resource_request,
        );
        let first_yielded = first.inspection().clone();
        let second_yielded = second.inspection().clone();
        let first = direct_recovery_cleanup(execution, first, &runtime, &bridge);
        let second = direct_recovery_cleanup(execution, second, &runtime, &bridge);

        let first = pending_direct_cleanup(first);
        let second = pending_direct_cleanup(second);
        assert_direct_pending_association(first.inspection(), &first_yielded);
        assert_direct_pending_association(second.inspection(), &second_yielded);
        assert_ne!(
            first.inspection().yielded_attempt_identity(),
            second.inspection().yielded_attempt_identity()
        );

        let second = match second.retry() {
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("rightful second direct cleanup retry must complete"),
        };
        let first = match first.retry() {
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("rightful first direct cleanup retry must complete"),
        };
        assert_direct_cleanup_association(first.inspection(), &first_yielded);
        assert_direct_cleanup_association(second.inspection(), &second_yielded);
    });
}

pub fn direct_recovery_cleanup(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    yielded: worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupRequired {
    let active_request = execution;

    {
        let fixture_phase_result = {
            let fixture_target = yielded;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
            fixture_target.readmit_same_runtime(&installed_fixture_phase, runtime, bridge)
        };
        match fixture_phase_result {
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
            worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
                recovery,
            ),
        ) => recovery.into_cleanup(),
        _ => panic!("restore panic must produce direct terminal cleanup"),
    }
    }
}

pub fn pending_direct_cleanup(
    cleanup: worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupRequired,
) -> worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupPending {
    std::thread::spawn(move || match cleanup.finish() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Pending(pending) => {
            pending
        }
        _ => panic!("foreign-thread direct cleanup must retain Bridge retry authority"),
    })
    .join()
    .expect("direct cleanup probe must return Pending")
}

pub fn assert_direct_pending_association(
    cleanup: &worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupPendingInspection,
    yielded: &worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRunInspection,
) {
    assert_eq!(
        cleanup.logical_run_identity(),
        yielded.logical_run_identity()
    );
    assert_eq!(
        cleanup.yielded_attempt_identity(),
        yielded.yielded_attempt_identity()
    );
    assert_eq!(
        cleanup.provider_session_identity(),
        yielded.provider_session_identity()
    );
    assert_eq!(
        cleanup.checkpoint().identity(),
        yielded.checkpoint().identity()
    );
    assert!(!cleanup.resource_plan_identity().is_empty());
    assert!(cleanup.bridge_cleanup_pending());
}

pub fn assert_direct_cleanup_association(
    cleanup: &worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionCleanupInspection,
    yielded: &worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRunInspection,
) {
    assert_eq!(
        cleanup.logical_run_identity(),
        yielded.logical_run_identity()
    );
    assert_eq!(
        cleanup.yielded_attempt_identity(),
        yielded.yielded_attempt_identity()
    );
    assert_eq!(
        cleanup.provider_session_identity(),
        yielded.provider_session_identity()
    );
    assert_eq!(
        cleanup.checkpoint().identity(),
        yielded.checkpoint().identity()
    );
    assert!(!cleanup.resource_plan_identity().is_empty());
    assert!(cleanup.resources_released());
    assert_eq!(cleanup.released_reservation_count(), 2);
}

#[test]
fn same_scope_peer_inspection_survives_denial_while_the_other_owner_cleans_up() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, second, bridge, runtime) = shared_yielded_peers(execution, resource_request);
        let first_inspection = first.inspection().clone();
        let second_inspection = second.inspection().clone();
        assert_eq!(
            first_inspection.operation_binding_identity(),
            second_inspection.operation_binding_identity()
        );
        assert_ne!(
            first_inspection.yielded_attempt_identity(),
            second_inspection.yielded_attempt_identity()
        );

        let foreign_bridge = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::managed_admission_context(&resource_request).bridge;
        let first = {
            let fixture_phase_result = {
                let fixture_target = first;
                let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
                fixture_target.readmit_same_runtime(
                    &installed_fixture_phase,
                    &runtime,
                    &foreign_bridge,
                )
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => {
                    denied.into_yielded()
                }
                _ => panic!("foreign Bridge must deny the exact direct yielded owner"),
            }
        };
        assert_eq!(first.inspection(), &first_inspection);

        let active = {
            let fixture_phase_result = {
                let fixture_target = first;
                let installed_fixture_phase = fixture_target.fixture_phase_for_test(active_request);
                fixture_target.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                worth_query_execution::facade::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted.into_active(),
                _ => panic!("rightful Bridge must readmit the returned direct owner"),
            }
        };
        let terminal = {
            let fixture_phase_result = {
                let fixture_target = active;
                let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
                fixture_target.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("rightfully readmitted direct owner must complete"),
            }
        }
        .into_running()
        .completed()
        .expect("completed direct owner must terminalize");
        assert!(terminal.cleanup().is_ok());

        let cleanup = complete_direct_yield_cleanup(second);
        assert_eq!(
            cleanup
                .checkpoint()
                .expect("ordinary cleanup must retain checkpoint evidence")
                .identity(),
            second_inspection.checkpoint().identity()
        );
        assert_eq!(
            cleanup.yielded_attempt_identity(),
            second_inspection.yielded_attempt_identity()
        );
        assert_eq!(
            cleanup.provider_session_identity(),
            second_inspection.provider_session_identity()
        );
        assert!(cleanup.resources_released());
        assert_eq!(cleanup.released_reservation_count(), 2);
    });
}

pub fn yield_peer(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    runtime: &WorthQueryExecutionRuntime,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    lower: &worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::managed_run::tests::causal_fixture::CausalManagedAdmissionContext,
    provider_support: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    binding_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun {
    let plan =
        admitted_plan_with_graph_support(binding_identity, 8, graph.role(), provider_support);
    let operation = direct_authority_with_graph(
        runtime,
        &plan,
        graph,
        WorthQueryOperationGraphAccess::Observe,
    );
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("shared peer resource attempt should start");
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&operation, attempt, lower.read_request(), resource_request)
        .expect("shared peer should admit")
        .start();
    let active = {
        let fixture_target = running;
        let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
        fixture_target.begin_graph_execution(
            &installed_fixture_phase,
            graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                binding_identity,
            ),
        )
    }
    .expect("shared peer provider should begin");
    let paused = {
        let fixture_phase_result = {
            let fixture_target = active;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("shared peer provider did not pause"),
        }
    };
    match paused.yield_run() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("shared peer should yield"),
    }
}

pub fn assert_cleanup_association(
    cleanup: &worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
    resource_identity: &str,
    provider_session_identity: &str,
) {
    let inspection = cleanup.inspection();
    assert_eq!(inspection.run_identity(), resource_identity);
    assert_eq!(
        inspection.provider_session_identity(),
        provider_session_identity
    );
    assert_eq!(
        inspection.provider_work().provider_session_identity(),
        provider_session_identity
    );
    assert!(inspection.resources_released());
    assert_eq!(inspection.released_reservation_count(), 2);
}

#[path = "interleaved_peers/mod.rs"]
pub mod interleaved_peers;
