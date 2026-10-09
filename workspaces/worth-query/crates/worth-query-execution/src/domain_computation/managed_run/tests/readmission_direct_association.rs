use super::yield_fixture::YieldProvider;
use super::*;

fn shared_yielded_peers(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
) -> (
    crate::domain_computation::WorthQueryYieldedDirectRun,
    crate::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    shared_yielded_peers_with_provider(execution, YieldProvider::installed(5))
}

fn shared_yielded_peers_with_provider(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
) -> (
    crate::domain_computation::WorthQueryYieldedDirectRun,
    crate::domain_computation::WorthQueryYieldedDirectRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, YieldProvider>(
            provider,
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
    .expect("shared graph authority should install");
    let runtime = installer
        .install(
            worth_query_installation::facade::WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .expect("shared execution runtime should install")
        .into_parts()
        .0;
    let lower = causal_fixture::managed_admission_context();
    let first = yield_peer(
        execution,
        &runtime,
        &graph,
        &lower,
        provider_support.clone(),
        "direct-peer",
    );
    let second = yield_peer(
        execution,
        &runtime,
        &graph,
        &lower,
        provider_support,
        "direct-peer",
    );
    (first, second, lower.bridge, runtime)
}

#[test]
fn cleanup_pending_peers_keep_exact_yielded_association_through_rightful_retry() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (first, second, bridge, runtime) = shared_yielded_peers_with_provider(
            execution,
            YieldProvider::checkpoint_restore_panic(5),
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
            crate::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("rightful second direct cleanup retry must complete"),
        };
        let first = match first.retry() {
            crate::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Complete(
                receipt,
            ) => receipt,
            _ => panic!("rightful first direct cleanup retry must complete"),
        };
        assert_direct_cleanup_association(first.inspection(), &first_yielded);
        assert_direct_cleanup_association(second.inspection(), &second_yielded);
    });
}

fn direct_recovery_cleanup(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: crate::domain_computation::WorthQueryYieldedDirectRun,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> crate::domain_computation::WorthQueryDirectReadmissionCleanupRequired {
    let active_request = execution;

    match yielded.readmit_same_runtime(active_request, runtime, bridge) {
        crate::domain_computation::WorthQueryDirectReadmissionOutcome::RecoveryRequired(
            crate::domain_computation::WorthQueryDirectReadmissionRecoveryRequired::TerminalCleanup(
                recovery,
            ),
        ) => recovery.into_cleanup(),
        _ => panic!("restore panic must produce direct terminal cleanup"),
    }
}

fn pending_direct_cleanup(
    cleanup: crate::domain_computation::WorthQueryDirectReadmissionCleanupRequired,
) -> crate::domain_computation::WorthQueryDirectReadmissionCleanupPending {
    std::thread::spawn(move || match cleanup.finish() {
        crate::domain_computation::WorthQueryDirectReadmissionCleanupOutcome::Pending(pending) => {
            pending
        }
        _ => panic!("foreign-thread direct cleanup must retain Bridge retry authority"),
    })
    .join()
    .expect("direct cleanup probe must return Pending")
}

fn assert_direct_pending_association(
    cleanup: &crate::domain_computation::WorthQueryDirectReadmissionCleanupPendingInspection,
    yielded: &crate::domain_computation::WorthQueryYieldedDirectRunInspection,
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

fn assert_direct_cleanup_association(
    cleanup: &crate::domain_computation::WorthQueryDirectReadmissionCleanupInspection,
    yielded: &crate::domain_computation::WorthQueryYieldedDirectRunInspection,
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
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, second, bridge, runtime) = shared_yielded_peers(execution);
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

        let foreign_bridge = causal_fixture::managed_admission_context().bridge;
        let first = match first.readmit_same_runtime(active_request, &runtime, &foreign_bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Denied(denied) => {
                denied.into_yielded()
            }
            _ => panic!("foreign Bridge must deny the exact direct yielded owner"),
        };
        assert_eq!(first.inspection(), &first_inspection);

        let active = match first.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("rightful Bridge must readmit the returned direct owner"),
        };
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("rightfully readmitted direct owner must complete"),
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

fn yield_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryExecutionRuntime,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    lower: &causal_fixture::CausalManagedAdmissionContext,
    provider_support: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    binding_identity: &str,
) -> crate::domain_computation::WorthQueryYieldedDirectRun {
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
        .admit_direct(&operation, attempt, lower.read_request())
        .expect("shared peer should admit")
        .start();
    let active = running
        .begin_graph_execution(
            execution,
            graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                binding_identity,
            ),
        )
        .expect("shared peer provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("shared peer provider did not pause"),
    };
    match paused.yield_run() {
        crate::domain_computation::WorthQueryDirectYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("shared peer should yield"),
    }
}

#[test]
fn interleaved_readmitted_peers_keep_attempt_ledger_and_release_authority_associated() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, second, bridge, runtime) = shared_yielded_peers(execution);
        let first_binding = first.inspection().operation_binding_identity().to_owned();
        let second_binding = second.inspection().operation_binding_identity().to_owned();

        let first = match first.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("first shared peer should readmit"),
        };
        let second = match second.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("second shared peer should readmit"),
        };
        let first_resource = first.resource_attempt_identity().to_owned();
        let second_resource = second.resource_attempt_identity().to_owned();
        let first_session = first.provider_session_identity().to_owned();
        let second_session = second.provider_session_identity().to_owned();
        let first_bridge_basis = first.bridge_basis_identity().to_owned();
        let second_bridge_basis = second.bridge_basis_identity().to_owned();
        let first_bridge_intent = worth_runtime_bridge::facade::BridgeManagedExecutionIntent::new(
            first_binding,
            first_resource.clone(),
        )
        .identity()
        .as_str()
        .to_owned();
        let second_bridge_intent = worth_runtime_bridge::facade::BridgeManagedExecutionIntent::new(
            second_binding,
            second_resource.clone(),
        )
        .identity()
        .as_str()
        .to_owned();
        assert_ne!(first_resource, second_resource);
        assert_ne!(first_session, second_session);
        assert_ne!(first_bridge_basis, second_bridge_basis);
        assert_ne!(first_bridge_intent, second_bridge_intent);

        let first = match first.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("first restored peer should complete"),
        }
        .into_running()
        .completed()
        .expect("first peer should terminalize");
        assert_eq!(
            first.provider_work().provider_session_identity(),
            first_session
        );
        let first = first.cleanup().expect("first peer should clean up");
        assert_cleanup_association(&first, &first_resource, &first_session);

        assert_eq!(second.retained_capacity_reservation_count(), 2);
        let second = match second.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("second peer must remain live after first cleanup"),
        }
        .into_running()
        .completed()
        .expect("second peer should terminalize");
        assert_eq!(
            second.provider_work().provider_session_identity(),
            second_session
        );
        let second = second.cleanup().expect("second peer should clean up");
        assert_cleanup_association(&second, &second_resource, &second_session);
    });
}

fn assert_cleanup_association(
    cleanup: &crate::domain_computation::WorthQueryDirectRunCleanupReceipt,
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
