use std::sync::Arc;

use serde_json::json;
use worth_server::{
    WorthServerAdmittedDurableProductMutation, WorthServerDurableProductMutationConclusion,
    WorthServerDurableProductMutationContract, WorthServerDurableProductMutationExecution,
    WorthServerDurableProductMutationExecutor, WorthServerDurableProductMutationRecoveryHandle,
    WorthServerProductAdapterCertificationCode, WorthServerProductAdapterExecutionError,
    WorthServerProductAdapterRegistry, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration, WorthServerProductAuthorityScope,
    WorthServerProductDurabilityCapability, WorthServerProductIdempotencyRetention,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationErrorMaps, WorthServerProductOperationExecutionBoundary,
    WorthServerProductOperationInput, WorthServerProductOperationPayload,
    WorthServerProductOperationSuccess, WorthServerProductOperationSupportSnapshot,
    WorthServerProductOperationSurfaceDenialCode, WorthServerProductResultContract,
    WorthServerScheduledProductOperation,
};

#[path = "support/durable_product_mutation/mod.rs"]
mod durable_support;

#[test]
fn adapter_cannot_return_a_result_under_an_undeclared_schema() {
    let declared_contract = result_contract("product.connection.result.v1");
    let adapter = Arc::new(WrongResultContractAdapter {
        returned_contract: result_contract("product.deployment.result.v1"),
    });
    let registration = WorthServerProductApplicationAdapterRegistration::new(
        "wrong-result-contract-adapter",
        adapter,
    )
    .with_operation(
        WorthServerProductOperationDeclaration::product_read(
            "product.connection.inspect",
            "product.connection.inspect.v1",
            declared_contract,
            WorthServerProductOperationBasisKind::DurableProductDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted(
                "connection-inspect-supported",
            ),
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    );
    let server = durable_support::build_server_with_registration(registration);
    let denial = durable_support::direct_session(&server)
        .product_operations()
        .execute(
            WorthServerProductOperationInput::new(
                "product.connection.inspect",
                WorthServerProductOperationPayload::json(
                    "product.connection.inspect.v1",
                    json!({ "connection_id": "host-7" }),
                ),
            )
            .with_basis_digest("basis:0"),
        )
        .expect_err("undeclared result schema must be denied");

    assert_eq!(
        denial.code(),
        WorthServerProductOperationSurfaceDenialCode::InvalidResultArtifact
    );
    assert_eq!(
        denial.facts().and_then(|facts| facts.execution_boundary()),
        Some(&WorthServerProductOperationExecutionBoundary::AdapterExecutionAttempted)
    );
}

#[test]
fn durable_declaration_without_atomic_executor_denies_registration() {
    let error = WorthServerProductAdapterRegistry::build(vec![durable_registration(None)])
        .expect_err("durable declaration without executor must fail registration");
    assert_eq!(
        error.certification_code(),
        Some(WorthServerProductAdapterCertificationCode::MissingDurableMutationExecutor)
    );
}

#[test]
fn weaker_durability_capability_cannot_satisfy_atomic_completion() {
    let executor: Arc<dyn WorthServerDurableProductMutationExecutor> = Arc::new(WeakExecutor);
    let error =
        WorthServerProductAdapterRegistry::build(vec![durable_registration(Some(executor))])
            .expect_err("weaker executor must fail registration");
    assert_eq!(
        error.certification_code(),
        Some(WorthServerProductAdapterCertificationCode::IncompatibleDurableMutationCapability)
    );
}

#[test]
fn route_inventory_binds_result_and_durability_contract_identity() {
    let executor = durable_support::TestDurableProductExecutor::default();
    let server = durable_support::build_server(&executor);
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

#[derive(Debug)]
struct WrongResultContractAdapter {
    returned_contract: WorthServerProductResultContract,
}

impl WorthServerProductApplicationAdapter for WrongResultContractAdapter {
    fn execute_with_lease(
        &self,
        operation: &WorthServerScheduledProductOperation,
        _lease: &worth_execution::ExecutionResourceLease<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError>,
        worth_execution::MapKernelStop,
    > {
        context.checkpoint(0)?;
        Ok(self.execute(operation))
    }

    fn execute(
        &self,
        _operation: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        durable_support::publish_schema_bound_json(
            "wrong-result",
            &self.returned_contract,
            self.returned_contract.schema().identity(),
            json!({ "deployment_id": "deploy-9" }),
        )
    }
}

struct WeakExecutor;

impl WorthServerDurableProductMutationExecutor for WeakExecutor {
    fn capability(&self) -> WorthServerProductDurabilityCapability {
        WorthServerProductDurabilityCapability::MutationWithoutAtomicCompletionV1
    }

    fn execute(
        &self,
        _attempt: &WorthServerAdmittedDurableProductMutation,
    ) -> WorthServerDurableProductMutationExecution {
        panic!("incompatible executor must never be admitted")
    }

    fn resolve(
        &self,
        _recovery: &WorthServerDurableProductMutationRecoveryHandle,
    ) -> WorthServerDurableProductMutationConclusion {
        WorthServerDurableProductMutationConclusion::failed(
            "incompatible_executor",
            "incompatible executor must never resolve",
        )
    }
}

fn durable_registration(
    executor: Option<Arc<dyn WorthServerDurableProductMutationExecutor>>,
) -> WorthServerProductApplicationAdapterRegistration {
    let registration = WorthServerProductApplicationAdapterRegistration::new(
        "durability-registration-boundary",
        Arc::new(WrongResultContractAdapter {
            returned_contract: result_contract("product.connection.result.v1"),
        }),
    )
    .with_operation(
        WorthServerProductOperationDeclaration::durable_product_mutation(
            "product.connection.upsert",
            "product.connection.upsert.v1",
            result_contract("product.connection.result.v1"),
            WorthServerProductOperationSupportSnapshot::production_admitted(
                "connection-upsert-supported",
            ),
            WorthServerDurableProductMutationContract::atomic(
                WorthServerProductAuthorityScope::new("connection")
                    .expect("test authority scope should validate"),
                WorthServerProductIdempotencyRetention::Indefinite,
            ),
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    );
    executor.map_or(registration.clone(), |executor| {
        registration.with_durable_mutation_executor(executor)
    })
}

fn result_contract(identity: &str) -> WorthServerProductResultContract {
    WorthServerProductResultContract::canonical_json(identity, 1, 1024)
        .expect("test result contract should validate")
}

fn is_sha256(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}
