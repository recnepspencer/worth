pub(crate) use worth_query_testkit_query::facade::query::suite::execution_resource_admission::provider_surfaces::*;

#[test]
fn conditional_provider_mismatch_denies_before_provider_contact() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = node(
        "resource-conditional-provider",
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQuerySemanticLocality::SourceRecord,
    );
    let installation = conditional_installation(&declaration);
    let contacts = Arc::new(AtomicUsize::new(0));
    let workspace = conditional_workspace_with(
        request_execution,
        "resource-conditional-provider-mismatch",
        declaration,
        installation,
        ResourceConditionalProvider {
            support: mismatched_fixture_support(),
            contacts: Arc::clone(&contacts),
        },
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();

    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        ReadExecutionInput::default(),
        execution_resource_request(),
        &workspace,
    ) else {
        panic!("conditional-provider support mismatch must deny")
    };

    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::
            CancellationSafePointUnsupported
    );
    assert!(denial.detail().contains("conditional node"));
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

#[test]
fn parallel_provider_mismatch_denies_before_provider_contact() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contacts = Arc::new(AtomicUsize::new(0));
    let workspace = workflow_workspace_with_parallel_provider(
        request_execution,
        "resource-parallel-provider-mismatch",
        ResourceParallelProvider {
            support: mismatched_fixture_support(),
            contacts: Arc::clone(&contacts),
        },
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap();

    let TransitionOutcome::Denied(denial) =
        bound.admit_workflow_resources(execution_resource_request(), &workspace)
    else {
        panic!("parallel-provider support mismatch must deny")
    };

    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::
            CancellationSafePointUnsupported
    );
    assert!(denial.detail().contains("parallel admission provider"));
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}
