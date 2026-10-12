//! domain predicate deferral is not reported as temporal or on demand.
use super::*;

#[test]
fn domain_predicate_deferral_is_not_reported_as_temporal_or_on_demand() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let node = conditional_node_result(
        "predicate-deferred",
        dependency,
        domain::WorthQueryConditionalEvaluationCondition::domain_specific::<GeometryCondition>([])
            .unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
    )
    .unwrap();
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(DeferredCondition);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        request_execution,
        "predicate-deferred",
        node,
        installation,
        CountedCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();

    let TransitionOutcome::Deferred(stopped) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("the installed predicate should defer")
    };
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DeferredByCondition
    );
    assert_eq!(
        host_conditional_signal_for_certification(&stopped.conditional_provenance()[0]),
        HostDecision::Deferred
    );
    assert_eq!(stopped.counters().conditional_condition_deferrals, 1);
    assert_eq!(stopped.counters().conditional_temporal_deferrals, 0);
    assert_eq!(stopped.counters().conditional_on_demand_deferrals, 0);
    assert_eq!(stopped.counters().conditional_compute_contacts, 0);
    assert_eq!(stopped.counters().conditional_decisions_delivered, 1);
}
