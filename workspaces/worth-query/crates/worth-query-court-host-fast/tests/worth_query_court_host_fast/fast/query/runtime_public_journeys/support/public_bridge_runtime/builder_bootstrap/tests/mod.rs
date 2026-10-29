pub(crate) use worth_query_worlds::facade::query::support::public_bridge_runtime::builder_bootstrap::tests::*;

#[test]
fn execution_runtime_requires_explicit_product_resources() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let product_world_resources = public_product_world_resources();
    let source = product_source("missing-resources", &product_world_resources);
    let bridge = bridge::public_bridge(&source);
    let result = harness
        .configure_runtime_builder(
            WorthQueryRuntime::builder(product_world_resources).relational_source_owner(source),
            bridge,
            public_bridge_aspect_contracts(),
            public_graph_support_profile(),
        )
        .build_backend_from_parts()
        .build(request_execution);

    let error = match result {
        Ok(_) => panic!("execution runtime must require Product World budgets"),
        Err(error) => error,
    };
    assert!(format!("{error:?}").contains("conditional_execution_resources"));
}

#[test]
fn execution_runtime_requires_an_installed_relational_product_source() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let product_world_resources = public_product_world_resources();
    let bridge_source = product_source("bridge-only-source", &product_world_resources);
    let bridge = bridge::public_bridge(&bridge_source);
    let result = harness
        .configure_runtime_builder(
            WorthQueryRuntime::builder(product_world_resources)
                .conditional_execution_resources(public_product_resources()),
            bridge,
            public_bridge_aspect_contracts(),
            public_graph_support_profile(),
        )
        .build_backend_from_parts()
        .build(request_execution);

    let error = match result {
        Ok(_) => panic!("execution runtime must require its Relational owner root"),
        Err(error) => error,
    };
    assert!(format!("{error:?}").contains("SourceNotInstalled"));
}

#[test]
fn execution_runtime_rejects_a_bridge_from_another_relational_owner() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let product_world_resources = public_product_world_resources();
    let installed_source = product_source("product-source", &product_world_resources);
    let foreign_bridge_source = product_source("foreign-bridge-source", &product_world_resources);
    let bridge = bridge::public_bridge(&foreign_bridge_source);
    let result = harness
        .configure_runtime_builder(
            WorthQueryRuntime::builder(product_world_resources)
                .relational_source_owner(installed_source)
                .conditional_execution_resources(public_product_resources()),
            bridge,
            public_bridge_aspect_contracts(),
            public_graph_support_profile(),
        )
        .build_backend_from_parts()
        .build(request_execution);

    let error = match result {
        Ok(_) => panic!("foreign Relational/Bridge pairing must fail closed"),
        Err(error) => error,
    };
    assert!(format!("{error:?}").contains("source"));
}
