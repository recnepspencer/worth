pub(crate) use worth_query_testkit_query::facade::query::suite::operation_native_access_generation::*;

#[test]
fn prior_generation_key_is_distinct_from_runtime_and_capability_drift() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = configured_runtime()
        .controlled_workspace(request_execution, "installed-native-key-generation")
        .unwrap();
    let prior_domain = workspace.domain(GeometryDomain).unwrap();
    let prior = bind(&workspace, &prior_domain);
    let (_prior_request, prior_key) = native_id_request(&prior);

    workspace.advance_domain_installation_generation().unwrap();
    let (current_domain, _) = workspace
        .rebind_domain(prior_domain.rebind_request())
        .unwrap()
        .into_parts();
    let current = bind(&workspace, &current_domain);
    let (current_request, current_key) = native_id_request(&current);
    let settled = current
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
        .consume_bound(current_request)
        .unwrap()
        .settle()
        .unwrap();

    assert!(settled.native_value(&current_key, 0).is_ok());
    let denial = settled.native_value(&prior_key, 0).unwrap_err();
    assert_eq!(
        denial.kind(),
        domain::WorthQueryNativeAccessDenialKind::AccessKeyInstallationGenerationMismatch
    );
    assert_eq!(denial.contract_identity(), prior_key.contract_identity());
    assert_eq!(denial.counters().authority_checks, 3);
    assert_eq!(denial.counters().indexed_accesses, 0);
    assert_eq!(denial.counters().refinement_checks, 0);
}
