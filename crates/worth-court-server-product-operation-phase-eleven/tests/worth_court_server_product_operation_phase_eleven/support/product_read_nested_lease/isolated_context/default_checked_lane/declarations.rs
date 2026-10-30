// A funded checked packet must never fall back to an adapter's unleased method.
pub(crate) use std::sync::Arc;
pub(crate) use worth_execution::ExecutionResourceLease;
pub(crate) use {
    worth_server::WorthServerProductAdapterExecutionError,
    worth_server::WorthServerProductApplicationAdapter,
    worth_server::WorthServerProductApplicationAdapterRegistration,
    worth_server::WorthServerProductOperationBasisKind,
    worth_server::WorthServerProductOperationDeclaration,
    worth_server::WorthServerProductOperationErrorMaps,
    worth_server::WorthServerProductOperationInput,
    worth_server::WorthServerProductOperationOutcome,
    worth_server::WorthServerProductOperationPayload,
    worth_server::WorthServerProductOperationSuccess,
    worth_server::WorthServerProductOperationSupportSnapshot,
    worth_server::WorthServerScheduledProductOperation,
    worth_server::WorthServerWorthNativeSession,
};
pub(crate) struct OrdinaryOnly;
impl WorthServerProductApplicationAdapter for OrdinaryOnly {
    fn execute(
        &self,
        _: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        panic!("a checked packet must never execute the unleased fallback");
    }
}
pub(crate) fn registration() -> WorthServerProductApplicationAdapterRegistration {
    worth_server::WorthServerProductApplicationAdapterRegistration::new("ordinary-only", std::sync::Arc::new(OrdinaryOnly))
        .with_operation(
            worth_server::WorthServerProductOperationDeclaration::product_read(
                "ordinary.only",
                "ordinary.payload.v1",
                worth_testkit_server::shared_support_product_adapter_phase_nine_fixture::result_contract("ordinary.result.v1"),
                worth_server::WorthServerProductOperationBasisKind::DurableProductDerived,
                worth_server::WorthServerProductOperationSupportSnapshot::production_admitted("ordinary-only"),
            )
            .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
        )
}
pub(crate) fn assert_refused(
    session: &WorthServerWorthNativeSession,
    lease: &ExecutionResourceLease<'_>,
) {
    let result = session
        .product_operations()
        .execute_shared_read_batch_with_lease(
            vec![worth_server::WorthServerProductOperationInput::new(
                "ordinary.only",
                worth_server::WorthServerProductOperationPayload::json(
                    "ordinary.payload.v1",
                    serde_json::json!({}),
                ),
            )
            .with_basis_digest("basis:r0")],
            lease,
        )
        .expect("funded packet reaches its adapter");
    assert_eq!(result.operations().len(), 1);
    let worth_server::WorthServerProductOperationOutcome::Denied(denial) =
        result.operations()[0].outcome()
    else {
        panic!("unimplemented checked lane must deny")
    };
    assert_eq!(denial.reason_key(), "product.checked_execution_unsupported");
}
