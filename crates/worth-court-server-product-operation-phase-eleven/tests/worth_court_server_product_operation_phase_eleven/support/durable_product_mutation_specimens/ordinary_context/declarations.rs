#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {serde_json::json, serde_json::Value};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation as durable_support};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver as request_driver};

pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server, worth_testkit_server::shared_support_durable_product_mutation::driver::execute, worth_testkit_server::shared_support_durable_product_mutation::driver::result_body, worth_testkit_server::shared_support_durable_product_mutation::runtime::session, worth_testkit_server::shared_support_durable_product_mutation::runtime::session_with_principal, worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor};
pub(crate) use {worth_testkit_server::shared_support_route_assembly_phase_twelve_request_driver::WorthServerRouteHttpTestDriver};

pub(crate) struct ProductMutationSpecimen {
    pub(crate) operation_name: &'static str,
    pub(crate) payload_schema: &'static str,
    pub(crate) body: Value,
    pub(crate) expected_schema: &'static str,
    pub(crate) expected_retention: &'static str,
}

pub(crate) fn product_mutation_specimens() -> [ProductMutationSpecimen; 3] {
    [
        ProductMutationSpecimen {
            operation_name: "product.host_connection.upsert",
            payload_schema: "product.host-connection.upsert.v1",
            body: json!({ "connection_id": "host-7", "endpoint": "https://host.test" }),
            expected_schema: "product.host-connection.result.v1",
            expected_retention: "indefinite",
        },
        ProductMutationSpecimen {
            operation_name: "product.manifest.admit",
            payload_schema: "product.manifest.admit.v1",
            body: json!({ "manifest_digest": "manifest:abc", "admitted": true }),
            expected_schema: "product.manifest.result.v1",
            expected_retention: "at-least-seconds:86400",
        },
        ProductMutationSpecimen {
            operation_name: "product.deployment.transition",
            payload_schema: "product.deployment.transition.v1",
            body: json!({ "deployment_id": "deploy-9", "phase": "active" }),
            expected_schema: "product.deployment.result.v1",
            expected_retention: "at-least-seconds:604800",
        },
    ]
}