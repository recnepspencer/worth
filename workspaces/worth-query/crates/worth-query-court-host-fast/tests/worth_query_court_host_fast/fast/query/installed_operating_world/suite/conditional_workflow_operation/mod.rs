pub(crate) use worth_query_worlds::facade::query::suite::conditional_workflow_operation::*;

pub(crate) mod advancement_custody;

#[test]
fn eligible_operation_condition_enters_the_run_before_any_stage_work() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let node = node(
        "workflow-operation-eligible",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&node);
    let captured = Arc::new(Mutex::new(None));
    let mut workspace = operation_conditional_workflow_workspace_with(
        request_execution,
        "workflow-operation-eligible",
        node,
        installation,
        CapturingWorkflowCompute(Arc::clone(&captured)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();

    let admitted = bound
        .admit_workflow_resources(
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();
    assert_eq!(admitted.resources().counters().runtime_authority_checks, 1);
    let run = admitted.start_workflow(&mut workspace).unwrap();

    let context = captured.lock().unwrap().take().unwrap();
    assert_eq!(context.run_identity.as_deref(), Some(run.identity()));
    assert!(!context.snapshot_identity.is_empty());
    assert_eq!(context.attempt, 1);
    assert_eq!(run.operation_conditional_provenance().len(), 1);
    assert_eq!(
        run.operation_conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::ComputedChanged
    );
    assert_eq!(run.counters().runtime_authority_checks, 0);
    assert_eq!(run.counters().conditional_compute_contacts, 1);
    assert_eq!(run.counters().stage_admission_checks, 0);
    assert_eq!(run.counters().stage_executor_contacts, 0);

    let trace = complete_standard_workflow(run, &mut workspace);
    assert_eq!(trace.operation_conditional_provenance().len(), 1);
    assert_eq!(trace.semantics().operation_conditional_path().len(), 1);
}

#[test]
fn deferred_operation_condition_returns_fresh_retry_authority_and_zero_stage_work() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let node = domain_condition_node("workflow-operation-deferred");
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(StaticCondition(
            worth_signal::facade::InstalledSignalConditionDecision::Deferred,
        ));
    let compute_contacts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut workspace = operation_conditional_workflow_workspace_with(
        request_execution,
        "workflow-operation-deferred",
        node,
        installation,
        CountedWorkflowCompute(Arc::clone(&compute_contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();

    let TransitionOutcome::Deferred(first) = bound
        .admit_workflow_resources(
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
    else {
        panic!("the operation-level condition must defer workflow creation")
    };
    assert_zero_stage_work(first.counters());
    assert_eq!(first.attempt(), 1);
    assert_eq!(
        first.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::DeferredByCondition
    );
    let first_identity = first.run_identity().to_owned();

    let TransitionOutcome::Deferred(second) = first.retry(&mut workspace) else {
        panic!("a retry must perform a fresh operation-level conditional attempt")
    };
    assert_zero_stage_work(second.counters());
    assert_eq!(second.attempt(), 2);
    assert_ne!(second.run_identity(), first_identity);
    assert_eq!(
        compute_contacts.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[test]
fn ineligible_operation_condition_cannot_create_a_workflow_run() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let node = domain_condition_node("workflow-operation-suppressed");
    let mut installation = conditional_installation(&node);
    installation.providers = worth_runtime_bridge::facade::BridgeConditionalProviderSet::new()
        .condition(StaticCondition(
            worth_signal::facade::InstalledSignalConditionDecision::Suppressed,
        ));
    let compute_contacts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut workspace = operation_conditional_workflow_workspace_with(
        request_execution,
        "workflow-operation-suppressed",
        node,
        installation,
        CountedWorkflowCompute(Arc::clone(&compute_contacts)),
    )
    .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap();

    let TransitionOutcome::Deferred(stopped) = bound
        .admit_workflow_resources(
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
    else {
        panic!("a suppressed operation-level condition must not mint a workflow run")
    };

    assert_zero_stage_work(stopped.counters());
    assert_eq!(
        stopped.conditional_provenance()[0].class(),
        domain::WorthQueryConditionalOutcomeClass::Suppressed
    );
    assert_eq!(
        compute_contacts.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}
