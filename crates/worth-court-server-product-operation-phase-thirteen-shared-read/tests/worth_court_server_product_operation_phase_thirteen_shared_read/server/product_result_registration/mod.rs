#[allow(unused_imports)]
use crate::product_result_registration::ordinary_context::*;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use std::sync::Arc;

use serde_json::json;
use worth_server::WorthServerDurableProductMutationExecutor;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_durable_product_mutation as durable_support;

#[test]
fn adapter_cannot_return_a_result_under_an_undeclared_schema() {
    let declared_contract = result_contract("product.connection.result.v1");
    let adapter = std::sync::Arc::new(WrongResultContractAdapter {
        returned_contract: result_contract("product.deployment.result.v1"),
    });
    let registration = worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "wrong-result-contract-adapter",
        adapter,
    )
    .with_operation(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "product.connection.inspect",
            "product.connection.inspect.v1",
            declared_contract,
            worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "connection-inspect-supported",
            ),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    );
    let server = worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server_with_registration(registration);
    let denial =
        worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::direct_session(
            &server,
        )
        .product_operations()
        .execute(
            worth_server::WorthServerProductOperationInput::new(
                "product.connection.inspect",
                worth_server::WorthServerProductOperationPayload::json(
                    "product.connection.inspect.v1",
                    json!({ "connection_id": "host-7" }),
                ),
            )
            .with_basis_digest("basis:0"),
        )
        .expect_err("undeclared result schema must be denied");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerProductOperationSurfaceDenialCode::InvalidResultArtifact
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.execution_boundary()),
        Some(
            &worth_server::WorthServerProductOperationExecutionBoundary::AdapterExecutionAttempted
        )
    );
}

#[test]
fn durable_declaration_without_atomic_executor_denies_registration() {
    let error =
        worth_server::WorthServerProductAdapterRegistry::build(vec![durable_registration(None)])
            .expect_err("durable declaration without executor must fail registration");
    assert_eq!(
        error.certification_code(),
        Some(worth_server::WorthServerProductAdapterCertificationCode::MissingDurableMutationExecutor)
    );
}

#[test]
fn weaker_durability_capability_cannot_satisfy_atomic_completion() {
    let executor: Arc<dyn WorthServerDurableProductMutationExecutor> =
        std::sync::Arc::new(WeakExecutor);
    let error = worth_server::WorthServerProductAdapterRegistry::build(vec![durable_registration(
        Some(executor),
    )])
    .expect_err("weaker executor must fail registration");
    assert_eq!(
        error.certification_code(),
        Some(worth_server::WorthServerProductAdapterCertificationCode::IncompatibleDurableMutationCapability)
    );
}

#[test]
fn route_inventory_binds_result_and_durability_contract_identity() {
    let executor = worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor::default();
    let server =
        worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server(
            &executor,
        );
    let inventory = server.route_inventory();
    let row = inventory
        .rows()
        .iter()
        .find(|row| row.operation_name() == Some("product.host_connection.upsert"))
        .expect("durable host route should be inventoried");

    assert!(row.result_contract_digest().is_some_and(is_sha256));
    assert!(row.durability_contract_digest().is_some_and(is_sha256));
    assert_ne!(
        row.result_contract_digest(),
        row.durability_contract_digest()
    );
}
