#[path = "support/test_bootstrap_policy.rs"]
mod test_bootstrap_policy;

use serde_json::json;
use worth_server::{
    WorthServerProductOperationExecutionBoundary, WorthServerProductOperationInput,
    WorthServerProductOperationPayload, WorthServerProductOperationSurfaceDenialCode,
};

#[path = "support/durable_product_mutation/mod.rs"]
mod durable_support;

use durable_support::{build_server, session, TestDurableProductExecutor};

#[test]
fn missing_durable_preconditions_never_reach_product_persistence() {
    let executor = TestDurableProductExecutor::default();
    let server = build_server(&executor);
    let operations = session(&server, "tenant-a", "workspace-42").product_operations();
    let payload = || {
        WorthServerProductOperationPayload::json(
            "product.host-connection.upsert.v1",
            json!({ "connection_id": "host-7" }),
        )
    };

    let missing_basis = operations
        .execute(
            WorthServerProductOperationInput::new("product.host_connection.upsert", payload())
                .with_idempotency_key(
                    worth_server::WorthServerProductIdempotencyKey::new("boundary-key")
                        .expect("test key should validate"),
                ),
        )
        .expect_err("durable mutation without a basis must deny");
    let missing_key = operations
        .execute(
            WorthServerProductOperationInput::new("product.host_connection.upsert", payload())
                .with_basis_digest("basis:0"),
        )
        .expect_err("durable mutation without an idempotency key must deny");

    for denial in [missing_basis, missing_key] {
        assert_eq!(
            denial.code(),
            WorthServerProductOperationSurfaceDenialCode::PreconditionDenied
        );
        assert_eq!(
            denial.facts().and_then(|facts| facts.execution_boundary()),
            Some(&WorthServerProductOperationExecutionBoundary::RejectedBeforeAdapterExecution)
        );
    }
    assert_eq!(executor.commit_count(), 0);
    assert_eq!(server.counters().durable_product_mutation_attempts, 0);
}
