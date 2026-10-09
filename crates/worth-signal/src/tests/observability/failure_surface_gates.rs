use crate::facade::{
    EvaluationRequestMode, SignalError, SignalGraph, SignalObservationRequest, SignalRuntime,
    SignalRuntimePolicy,
};
use crate::tests::support::ASPECT_A;

fn fail_transaction(runtime: &mut SignalRuntime<(), (), (), (), ()>) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let node = runtime.graph_mut().node().build();
    let mut context = ();
    let result = runtime.transaction(request_execution, &mut context, |transaction| {
        transaction.mark_dirty(node, ASPECT_A)?;
        transaction.evaluate_with_plan(
            node,
            &|_view| {
                Err::<crate::logic::evaluation::EvaluationOutput, _>(SignalError::internal(
                    "observation gate failure",
                ))
            },
            EvaluationRequestMode::Default,
        )?;
        Ok(())
    });
    assert!(result.is_err());
}

#[test]
fn operational_failure_and_rollback_details_stay_unmaterialized() {
    let mut runtime = SignalRuntime::operational(SignalGraph::new());
    fail_transaction(&mut runtime);

    assert!(runtime.observe().latest_failure_diagnostics().is_none());
    assert!(runtime.observe().latest_rollback_diagnostics().is_none());
    let branch = runtime.observe().current_branch().id;
    assert!(runtime
        .observe()
        .replay_for_branch(branch)
        .frames
        .is_empty());
}

#[test]
fn development_failure_and_rollback_details_are_materialized_when_selected() {
    let mut runtime = SignalRuntime::development(SignalGraph::new());
    fail_transaction(&mut runtime);

    assert!(runtime.observe().latest_failure_diagnostics().is_some());
    assert!(runtime.observe().latest_rollback_diagnostics().is_some());
    let branch = runtime.observe().current_branch().id;
    assert!(!runtime
        .observe()
        .replay_for_branch(branch)
        .frames
        .is_empty());
}

#[test]
fn rich_retention_does_not_override_an_on_demand_surface_gate() {
    let rich_on_demand = SignalRuntimePolicy::development()
        .with_history_details(true)
        .with_observation_activation(worth_foundational::ObservationActivationProfile::OnDemand);

    let mut counters_only = SignalRuntime::operational(SignalGraph::new());
    counters_only.set_runtime_policy(rich_on_demand);
    let counters_session = counters_only
        .begin_observation_session(SignalObservationRequest::counters())
        .unwrap();
    fail_transaction(&mut counters_only);
    assert!(counters_only
        .observe()
        .latest_failure_diagnostics()
        .is_none());
    counters_only
        .cancel_observation_session(&counters_session)
        .unwrap();

    let mut facts_selected = SignalRuntime::operational(SignalGraph::new());
    facts_selected.set_runtime_policy(rich_on_demand);
    let facts_session = facts_selected
        .begin_observation_session(SignalObservationRequest::facts())
        .unwrap();
    fail_transaction(&mut facts_selected);
    assert!(facts_selected
        .observe()
        .latest_failure_diagnostics()
        .is_some());
    facts_selected
        .cancel_observation_session(&facts_session)
        .unwrap();
}
