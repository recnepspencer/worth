#[allow(unused_imports)]
use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

use {serde_json::json};
use {worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerProductOperationSurfaceDenialCode};

#[allow(unused_imports)]
use {worth_testkit_server::shared_support_durable_product_mutation as durable_support};

use {worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server, worth_testkit_server::shared_support_durable_product_mutation::runtime::session, worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor};

#[test]
fn missing_durable_preconditions_never_reach_product_persistence() {
    let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let operations = session(&server, "tenant-a", "workspace-42").product_operations();
    let payload = || {
        worth_server::WorthServerProductOperationPayload::json(
            "product.host-connection.upsert.v1",
            json!({ "connection_id": "host-7" }),
        )
    };

    let missing_basis = operations
        .execute(
            worth_server::WorthServerProductOperationInput::new("product.host_connection.upsert", payload())
                .with_idempotency_key(
                    worth_server::WorthServerProductIdempotencyKey::new("boundary-key")
                        .expect("test key should validate"),
                ),
        )
        .expect_err("durable mutation without a basis must deny");
    let missing_key = operations
        .execute(
            worth_server::WorthServerProductOperationInput::new("product.host_connection.upsert", payload())
                .with_basis_digest("basis:0"),
        )
        .expect_err("durable mutation without an idempotency key must deny");

    for denial in [missing_basis, missing_key] {
        assert_eq!(
            denial.code(),
            worth_server::WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
        );
        assert_eq!(
            denial.facts().and_then(|facts| facts.execution_boundary()),
            Some(&worth_server::WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
        );
    }
    assert_eq!(executor.commit_count(), 0);
    assert_eq!(server.counters().durable_product_mutation_attempts, 0);
}
