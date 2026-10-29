pub(crate) use worth_query_testkit_query::facade::query::suite::operation_native_access::*;

#[test]
fn installed_projection_key_borrows_the_exact_foundational_value_in_constant_work() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution, "installed-native-access", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let (request, key) = native_id_request(bound.consumer_projection_contract().unwrap());

    let settled = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap();
    let access = settled.native_value(&key, 0).unwrap();

    assert_eq!(
        access.fact().as_interned_string(),
        Ok(&InternedString::Raw("synthetic-anchor".into()))
    );
    assert_eq!(access.counters().indexed_accesses, 1);
    assert_eq!(access.counters().refinement_checks, 1);
    assert_eq!(access.counters().fact_scans, 0);
    assert_eq!(access.counters().row_scans, 0);
    assert_eq!(access.counters().path_parses, 0);
}
