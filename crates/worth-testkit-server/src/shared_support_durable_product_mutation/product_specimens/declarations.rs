pub use {std::sync::Arc};

pub use {worth_server::WorthServerProductAdapterExecutionError, worth_server::WorthServerProductApplicationAdapter, worth_server::WorthServerProductApplicationAdapterRegistration, worth_server::WorthServerProductAuthorityScope, worth_server::WorthServerProductIdempotencyRetention, worth_server::WorthServerProductOperationDeclaration, worth_server::WorthServerProductOperationErrorMaps, worth_server::WorthServerProductOperationSuccess, worth_server::WorthServerProductOperationSupportSnapshot, worth_server::WorthServerProductResultContract, worth_server::WorthServerScheduledProductOperation};

pub use {crate::shared_support_durable_product_mutation::executor::TestDurableProductExecutor};

#[derive(Debug)]
pub struct NeverCalledAdapter;

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
    worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "durable-product-specimen",
        std::sync::Arc::new(NeverCalledAdapter),
    )
    .with_durable_mutation_executor(std::sync::Arc::new(executor))
    .with_operations([
        declaration(
            "product.host_connection.upsert",
            "product.host-connection.upsert.v1",
            "product.host-connection.result.v1",
            "host-connection",
            worth_server::WorthServerProductIdempotencyRetention::Indefinite,
            "durable-supported",
        ),
        declaration(
            "product.manifest.admit",
            "product.manifest.admit.v1",
            "product.manifest.result.v1",
            "admitted-manifest",
            worth_server::WorthServerProductIdempotencyRetention::at_least_seconds(86_400)
                .expect("manifest retention should validate"),
            "durable-supported",
        ),
        declaration(
            "product.deployment.transition",
            "product.deployment.transition.v1",
            "product.deployment.result.v1",
            "deployment",
            worth_server::WorthServerProductIdempotencyRetention::at_least_seconds(604_800)
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
    worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "versioned-host-connection-specimen",
        std::sync::Arc::new(NeverCalledAdapter),
    )
    .with_durable_mutation_executor(std::sync::Arc::new(executor))
    .with_operation(declaration(
        "product.host_connection.upsert",
        payload_schema,
        "product.host-connection.result.v1",
        "host-connection",
        worth_server::WorthServerProductIdempotencyRetention::Indefinite,
        support_row,
    ))
}

pub fn declaration(
    operation_name: &str,
    payload_schema: &str,
    result_schema: &str,
    authority_scope: &str,
    idempotency_retention: WorthServerProductIdempotencyRetention,
    support_row: &str,
) -> WorthServerProductOperationDeclaration {
    worth_server::WorthServerProductOperationDeclaration::durable_product_mutation(
        operation_name,
        payload_schema,
        worth_server::WorthServerProductResultContract::canonical_json(result_schema, 1, 16 * 1024)
            .expect("test result contract should validate"),
        worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(support_row),
        worth_server::WorthServerDurableProductMutationContract::atomic(
            worth_server::WorthServerProductAuthorityScope::new(authority_scope)
                .expect("test authority scope should validate"),
            idempotency_retention,
        ),
    )
    .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough())
}