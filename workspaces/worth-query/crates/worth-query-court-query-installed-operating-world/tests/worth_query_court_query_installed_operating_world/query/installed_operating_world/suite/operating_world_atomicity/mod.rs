pub(crate) use worth_query_testkit_query::facade::query::suite::operating_world_atomicity::*;

#[test]
fn primary_and_separate_mutation_requires_declared_compensation() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contacts = Arc::new(AtomicUsize::new(0));
    let uncompensated = runtime(
        request_execution,
        Arc::clone(&contacts),
        "mixed-uncompensated",
    );
    let installed = uncompensated.domain(GeometryDomain).unwrap();
    let denial = match uncompensated
        .prepare_mutation_operating_world(uncompensated.current_world())
        .unwrap()
        .family(MutationFamily)
        .bind(&installed, WorkflowMutation)
    {
        Ok(_) => panic!("separate commit authority cannot cover primary graph mutation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::CompensationUndeclared
    );
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}
