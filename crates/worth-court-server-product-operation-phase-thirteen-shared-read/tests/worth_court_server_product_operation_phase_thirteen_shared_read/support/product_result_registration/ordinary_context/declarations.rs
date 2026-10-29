#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {std::sync::Arc};

pub(crate) use {serde_json::json};
pub(crate) use {worth_server::WorthServerAdmittedDurableProductMutation, worth_server::WorthServerDurableProductMutationConclusion, worth_server::WorthServerDurableProductMutationContract, worth_server::WorthServerDurableProductMutationExecution, worth_server::WorthServerDurableProductMutationExecutor, worth_server::WorthServerDurableProductMutationRecoveryHandle, worth_server::WorthServerProductAdapterCertificationCode, worth_server::WorthServerProductAdapterExecutionError, worth_server::WorthServerProductAdapterRegistry, worth_server::WorthServerProductApplicationAdapter, worth_server::WorthServerProductApplicationAdapterRegistration, worth_server::WorthServerProductAuthorityScope, worth_server::WorthServerProductDurabilityCapability, worth_server::WorthServerProductIdempotencyRetention, worth_server::WorthServerProductOperationBasisKind, worth_server::WorthServerProductOperationDeclaration, worth_server::WorthServerProductOperationErrorMaps, worth_server::WorthServerProductOperationExecutionBoundary, worth_server::WorthServerProductOperationInput, worth_server::WorthServerProductOperationPayload, worth_server::WorthServerProductOperationSuccess, worth_server::WorthServerProductOperationSupportSnapshot, worth_server::WorthServerProductOperationSurfaceDenialCode, worth_server::WorthServerProductResultContract, worth_server::WorthServerScheduledProductOperation};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_durable_product_mutation as durable_support};

#[derive(Debug)]
pub(crate) struct WrongResultContractAdapter {
    pub(crate) returned_contract: WorthServerProductResultContract,
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
        worth_testkit_server::shared_support_product_result_schema_bound_json::publish_schema_bound_json(
            "wrong-result",
            &self.returned_contract,
            self.returned_contract.schema().identity(),
            json!({ "deployment_id": "deploy-9" }),
        )
    }
}

pub(crate) struct WeakExecutor;

impl WorthServerDurableProductMutationExecutor for WeakExecutor {
    fn capability(&self) -> WorthServerProductDurabilityCapability {
        worth_server::WorthServerProductDurabilityCapability::MutationWithoutAtomicCompletionV1
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
        worth_server::WorthServerDurableProductMutationConclusion::failed(
            "incompatible_executor",
            "incompatible executor must never resolve",
        )
    }
}

pub(crate) fn durable_registration(
    executor: Option<Arc<dyn WorthServerDurableProductMutationExecutor>>,
) -> WorthServerProductApplicationAdapterRegistration {
    let registration = worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "durability-registration-boundary",
        std::sync::Arc::new(WrongResultContractAdapter {
            returned_contract: result_contract("product.connection.result.v1"),
        }),
    )
    .with_operation(
        worth_server::WorthServerProductOperationDeclaration::durable_product_mutation(
            "product.connection.upsert",
            "product.connection.upsert.v1",
            result_contract("product.connection.result.v1"),
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "connection-upsert-supported",
            ),
            worth_server::WorthServerDurableProductMutationContract::atomic(
                worth_server::WorthServerProductAuthorityScope::new("connection")
                    .expect("test authority scope should validate"),
                worth_server::WorthServerProductIdempotencyRetention::Indefinite,
            ),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    );
    executor.map_or(registration.clone(), |executor| {
        registration.with_durable_mutation_executor(executor)
    })
}

pub(crate) fn result_contract(identity: &str) -> WorthServerProductResultContract {
    worth_server::WorthServerProductResultContract::canonical_json(identity, 1, 1024)
        .expect("test result contract should validate")
}

pub(crate) fn is_sha256(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}