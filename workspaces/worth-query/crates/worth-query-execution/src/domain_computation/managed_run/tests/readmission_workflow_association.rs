use std::collections::BTreeMap;
use std::sync::Arc;

use super::yield_fixture::YieldProvider;
use super::*;

fn shared_yielded_workflow_peers(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    shared_yielded_workflow_peers_with_provider(execution, YieldProvider::installed(5))
}

pub(super) fn shared_yielded_workflow_peers_with_provider(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    provider: YieldProvider,
) -> (
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    crate::domain_computation::WorthQueryYieldedWorkflowRun,
    RuntimeBridge,
    WorthQueryExecutionRuntime,
) {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let provider_anchor = Arc::new(
        crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor::install::<ManagedGraph, _>(
            provider,
        ),
    );
    let provider_support = provider_anchor.resource_support().clone();
    let graph = super::workflow_provider_steps::installed_graph(
        &installer,
        "workflow-association-graph",
        provider_anchor,
    );
    let runtime =
        super::workflow_provider_steps::installed_runtime(installer, "workflow association");
    let lower = causal_fixture::managed_admission_context();
    let first = yield_peer(
        execution,
        &runtime,
        &graph,
        &lower,
        provider_support.clone(),
        "workflow-peer",
    );
    let second = yield_peer(
        execution,
        &runtime,
        &graph,
        &lower,
        provider_support,
        "workflow-peer",
    );
    (first, second, lower.bridge, runtime)
}

fn yield_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    runtime: &WorthQueryExecutionRuntime,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    lower: &causal_fixture::CausalManagedAdmissionContext,
    provider_support: worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
    label: &str,
) -> crate::domain_computation::WorthQueryYieldedWorkflowRun {
    let operation_resources =
        crate::domain_computation::provider_session::admitted_yield_plan("workflow-association", 8);
    let stage_resources = admitted_plan_with_graph_support(
        "workflow-association:producer",
        8,
        graph.role(),
        provider_support,
    );
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_resources,
        BTreeMap::from([("producer".to_owned(), stage_resources)]),
    );
    let operation = workflow_authority_with_stage_graph(
        runtime,
        &resources,
        "producer",
        graph,
        WorthQueryOperationGraphAccess::Observe,
    );
    let attempt = runtime
        .start_workflow_resource_attempt(&operation, resources)
        .expect("shared workflow peer resource attempt should start");
    let running = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&operation, attempt, lower.read_request())
        .expect("shared workflow peer should admit")
        .start()
        .expect("shared workflow peer should start");
    let active = running
        .begin_stage_graph_execution(
            execution,
            "producer",
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, label),
        )
        .expect("shared workflow provider should begin");
    let paused = match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused,
        _ => panic!("shared workflow peer provider did not pause"),
    };
    match paused.yield_run() {
        crate::domain_computation::WorthQueryWorkflowYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("shared workflow peer should yield"),
    }
}

#[test]
fn interleaved_workflow_peers_keep_session_ledger_artifact_and_lower_bases_associated() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (first, second, bridge, runtime) = shared_yielded_workflow_peers(execution);
        let first_binding = first.inspection().operation_binding_identity().to_owned();
        let second_binding = second.inspection().operation_binding_identity().to_owned();
        let first_logical = first.inspection().logical_run_identity().to_owned();
        let second_logical = second.inspection().logical_run_identity().to_owned();
        let first_generation = first
            .inspection()
            .artifact_evidence()
            .production_generation();
        let second_generation = second
            .inspection()
            .artifact_evidence()
            .production_generation();
        assert_ne!(first_logical, second_logical);

        let first = readmit(execution, first, &runtime, &bridge);
        let second = readmit(execution, second, &runtime, &bridge);
        let first_session = first.provider_session_identity().to_owned();
        let second_session = second.provider_session_identity().to_owned();
        let first_resource = first.resource_attempt_identity().to_owned();
        let second_resource = second.resource_attempt_identity().to_owned();
        let first_bridge = first.bridge_basis_identity().to_owned();
        let second_bridge = second.bridge_basis_identity().to_owned();
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
        let first_artifact = first.artifact_evidence();
        let second_artifact = second.artifact_evidence();
        assert_ne!(first_session, second_session);
        assert_ne!(first_resource, second_resource);
        assert_ne!(first_bridge, second_bridge);
        assert_ne!(first_bridge_intent, second_bridge_intent);
        assert_eq!(first.logical_run_identity(), first_logical);
        assert_eq!(second.logical_run_identity(), second_logical);
        assert_eq!(first_artifact.production_generation(), first_generation + 1);
        assert_eq!(
            second_artifact.production_generation(),
            second_generation + 1
        );

        let first = complete(execution, first);
        assert_eq!(
            first.provider_work().provider_session_identity(),
            first_session
        );
        let first = cleanup(first);
        assert_cleanup_association(&first, &first_resource, &first_session);

        assert_eq!(second.retained_capacity_reservation_count(), 3);
        let second = complete(execution, second);
        assert_eq!(
            second.provider_work().provider_session_identity(),
            second_session
        );
        let second = cleanup(second);
        assert_cleanup_association(&second, &second_resource, &second_session);
    });
}

#[test]
fn foreign_bridge_denial_preserves_both_workflow_peers_without_fresh_query_work() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, second, bridge, runtime) = shared_yielded_workflow_peers(execution);
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
            crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Denied(denied) => {
                assert_eq!(
                denied.kind(),
                crate::domain_computation::WorthQueryWorkflowReadmissionDenialKind::
                    BridgeReadmissionDenied
            );
                let query = denied.readmission_evidence().query_counters();
                assert_eq!(query.fresh_resource_attempt_count(), 0);
                assert_eq!(query.provider_restore_attempt_count(), 0);
                assert_eq!(query.artifact_generation_attempt_count(), 0);
                denied.into_yielded()
            }
            _ => panic!("foreign Bridge must deny before workflow restamping"),
        };
        assert_eq!(first.inspection(), &first_inspection);

        let first = readmit(execution, first, &runtime, &bridge);
        let first = cleanup(complete(execution, first));
        assert_ne!(
            first.inspection().provider_session_identity(),
            first_inspection.provider_session_identity()
        );
        let second = match second.cleanup() {
            crate::domain_computation::WorthQueryWorkflowYieldCleanupOutcome::Complete(receipt) => {
                receipt
            }
            _ => panic!("artifact-free workflow peer must clean up directly"),
        };
        assert_eq!(
            second.inspection().checkpoint().identity(),
            second_inspection.checkpoint().identity()
        );
        assert_eq!(
            second.inspection().yielded_attempt_identity(),
            second_inspection.yielded_attempt_identity()
        );
        assert_eq!(
            second.inspection().provider_session_identity(),
            second_inspection.provider_session_identity()
        );
        assert!(second.inspection().resources_released());
        assert_eq!(second.inspection().released_reservation_count(), 3);
    });
}

fn readmit(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    yielded: crate::domain_computation::WorthQueryYieldedWorkflowRun,
    runtime: &WorthQueryExecutionRuntime,
    bridge: &RuntimeBridge,
) -> crate::domain_computation::WorthQueryActiveWorkflowGraphExecution {
    let active_request = execution;

    match yielded.readmit_same_runtime(active_request, runtime, bridge) {
        crate::domain_computation::WorthQueryWorkflowReadmissionOutcome::Readmitted(readmitted) => {
            let query = readmitted.readmission_evidence().query_counters();
            assert_eq!(query.fresh_resource_attempt_count(), 1);
            assert_eq!(query.provider_restore_attempt_count(), 1);
            assert_eq!(query.artifact_generation_attempt_count(), 1);
            assert_eq!(query.artifact_generation_commit_count(), 1);
            readmitted.into_active()
        }
        _ => panic!("rightful workflow peer should readmit"),
    }
}

fn complete(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    active: crate::domain_computation::WorthQueryActiveWorkflowGraphExecution,
) -> crate::domain_computation::WorthQueryWorkflowRunTerminal {
    match active.advance(execution) {
        WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion,
        _ => panic!("restored workflow peer should complete"),
    }
    .into_running()
    .completed()
    .expect("completed workflow peer should terminalize")
}

fn cleanup(
    terminal: crate::domain_computation::WorthQueryWorkflowRunTerminal,
) -> crate::domain_computation::WorthQueryWorkflowRunCleanupReceipt {
    match terminal.cleanup() {
        WorthQueryWorkflowRunCleanupOutcome::Complete(receipt) => receipt,
        _ => panic!("artifact-free workflow peer should clean up"),
    }
}

fn assert_cleanup_association(
    cleanup: &crate::domain_computation::WorthQueryWorkflowRunCleanupReceipt,
    resource: &str,
    session: &str,
) {
    let inspection = cleanup.inspection();
    assert_eq!(inspection.run_identity(), resource);
    assert_eq!(inspection.provider_session_identity(), session);
    assert_eq!(
        inspection.provider_work().provider_session_identity(),
        session
    );
    assert!(inspection.resources_released());
    assert_eq!(inspection.released_reservation_count(), 3);
}
