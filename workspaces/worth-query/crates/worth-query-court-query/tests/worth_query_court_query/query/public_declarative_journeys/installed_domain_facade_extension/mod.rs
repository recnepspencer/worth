pub(crate) use worth_query_testkit_query::facade::query::public_declarative_journeys::installed_domain_facade_extension::*;

#[test]
fn downstream_extension_preserves_the_generic_installed_capability_artifact() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = workspace(request_execution, "installed-domain-facade-extension");
    let handle = workspace
        .domain(PublicInstalledDomain)
        .expect("one installed handle lookup should admit");

    let generic = handle
        .declarations_in(&workspace, ConsumerOperatingContext)
        .expect("generic installed context should admit");
    let extended = handle
        .consumer_context(&workspace)
        .expect("downstream extension should lower through the same handle");

    assert_eq!(generic.handle_identity(), extended.handle_identity());
    assert_eq!(
        generic.installed_authority().witness_identity(),
        extended.installed_authority().witness_identity()
    );

    let runtime = workspace.into_runtime();
    let receipt = runtime
        .domain_installation_receipt(PublicInstalledDomain)
        .expect("the one package installation should retain a receipt");
    assert_eq!(receipt.construction_counters().package_lowerings(), 1);
    assert_eq!(receipt.construction_counters().derived_index_builds(), 1);
    assert_eq!(
        runtime
            .domain_installation_lookup_counters()
            .handle_lookups(),
        1
    );
    assert_eq!(
        runtime
            .domain_installation_lookup_counters()
            .package_content_scans(),
        0
    );
}
