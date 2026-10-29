pub(crate) use worth_query_worlds::facade::query::suite::conditional_failure_accounting::*;

#[test]
fn failed_conditional_compute_retains_exact_lower_runtime_work() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = node(
        "failed-compute-accounting",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&declaration);
    let contacts = Arc::new(AtomicUsize::new(0));
    let mut workspace = conditional_workspace_with(
        request_execution,
        "failed-compute-accounting",
        declaration,
        installation,
        FailingCompute(Arc::clone(&contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();

    let TransitionOutcome::Failed(denial) = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    else {
        panic!("the installed conditional compute failure must remain a checked failure")
    };

    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert!(matches!(
        denial.kind(),
        domain::WorthQueryBoundExecutionDenialKind::ConditionalExecution(
            domain::WorthQueryConditionalExecutionDenialKind::Bridge(
                worth_runtime_bridge::facade::BridgeConditionalDenialKind::SignalExecution(
                    worth_runtime_bridge::facade::BridgeSignalDenial::Error(native)
                )
            )
        ) if native == &worth_signal::facade::SignalError::invalid_input("declared compute failed")
    ));
    assert_eq!(denial.counters().conditional_dependency_checks, 1);
    assert_eq!(denial.counters().conditional_condition_checks, 1);
    assert_eq!(denial.counters().conditional_compute_contacts, 1);
    assert_eq!(denial.counters().conditional_semantic_changes, 0);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().executor_contacts, 0);
}
