use std::sync::Arc;

use worth_server::{
    WorthServerProductAdapterExecutionError, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration, WorthServerProductAuthorityScope,
    WorthServerProductIdempotencyRetention, WorthServerProductOperationDeclaration,
    WorthServerProductOperationErrorMaps, WorthServerProductOperationSuccess,
    WorthServerProductOperationSupportSnapshot, WorthServerProductResultContract,
    WorthServerScheduledProductOperation,
};

use super::TestDurableProductExecutor;

#[derive(Debug)]
struct NeverCalledAdapter;

impl WorthServerProductApplicationAdapter for NeverCalledAdapter {
    fn execute_with_lease(
        &self,
        _operation: &WorthServerScheduledProductOperation,
        _lease: &worth_execution::ExecutionResourceLease<'_>,
        _context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<
        Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError>,
        worth_execution::MapKernelStop,
    > {
        panic!("durable product mutations cannot enter a shared-read lease")
    }

    fn execute(
        &self,
        _operation: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        panic!("durable product mutations must execute through the durable executor")
    }
}

pub fn registration(
    executor: TestDurableProductExecutor,
) -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new(
        "durable-product-specimen",
        Arc::new(NeverCalledAdapter),
    )
    .with_durable_mutation_executor(Arc::new(executor))
    .with_operations([
        declaration(
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            "product.host-connection.result.v1",
            "host-connection",
            WorthServerProductIdempotencyRetention::Indefinite,
            "durable-supported",
        ),
        declaration(
            "product.manifest.admit",
            "product.manifest.admit.v1",
            "product.manifest.result.v1",
            "admitted-manifest",
            WorthServerProductIdempotencyRetention::at_least_seconds(86_400)
                .expect("manifest retention should validate"),
            "durable-supported",
        ),
        declaration(
            "product.deployment.transition",
            "product.deployment.transition.v1",
            "product.deployment.result.v1",
            "deployment",
            WorthServerProductIdempotencyRetention::at_least_seconds(604_800)
                .expect("deployment retention should validate"),
            "durable-supported",
        ),
    ])
}

pub fn host_registration(
    executor: TestDurableProductExecutor,
    payload_schema: &str,
) -> WorthServerProductApplicationAdapterRegistration {
    host_registration_with_support(executor, payload_schema, "durable-supported")
}

pub fn host_registration_with_support(
    executor: TestDurableProductExecutor,
    payload_schema: &str,
    support_row: &str,
) -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new(
        "versioned-host-connection-specimen",
        Arc::new(NeverCalledAdapter),
    )
    .with_durable_mutation_executor(Arc::new(executor))
    .with_operation(declaration(
        "product.host_connection.upsert",
        payload_schema,
        "product.host-connection.result.v1",
        "host-connection",
        WorthServerProductIdempotencyRetention::Indefinite,
        support_row,
    ))
}

fn declaration(
    operation_name: &str,
    payload_schema: &str,
    result_schema: &str,
    authority_scope: &str,
    idempotency_retention: WorthServerProductIdempotencyRetention,
    support_row: &str,
) -> WorthServerProductOperationDeclaration {
    WorthServerProductOperationDeclaration::durable_product_mutation(
        operation_name,
        payload_schema,
        WorthServerProductResultContract::canonical_json(result_schema, 1, 16 * 1024)
            .expect("test result contract should validate"),
        WorthServerProductOperationSupportSnapshot::production_admitted(support_row),
        worth_server::WorthServerDurableProductMutationContract::atomic(
            WorthServerProductAuthorityScope::new(authority_scope)
                .expect("test authority scope should validate"),
            idempotency_retention,
        ),
    )
    .with_error_map(WorthServerProductOperationErrorMaps::passthrough())
}
