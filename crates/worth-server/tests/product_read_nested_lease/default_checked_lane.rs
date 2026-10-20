//! A funded checked packet must never fall back to an adapter's unleased method.
use std::sync::Arc;
use worth_execution::ExecutionResourceLease;
use worth_server::{
    WorthServerProductAdapterExecutionError, WorthServerProductApplicationAdapter,
    WorthServerProductApplicationAdapterRegistration, WorthServerProductOperationBasisKind,
    WorthServerProductOperationDeclaration, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationInput, WorthServerProductOperationOutcome,
    WorthServerProductOperationPayload, WorthServerProductOperationSuccess,
    WorthServerProductOperationSupportSnapshot, WorthServerScheduledProductOperation,
    WorthServerWorthNativeSession,
};
struct OrdinaryOnly;
impl WorthServerProductApplicationAdapter for OrdinaryOnly {
    fn execute(
        &self,
        _: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        panic!("a checked packet must never execute the unleased fallback");
    }
}
pub(super) fn registration() -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new("ordinary-only", Arc::new(OrdinaryOnly))
        .with_operation(
            WorthServerProductOperationDeclaration::product_read(
                "ordinary.only",
                "ordinary.payload.v1",
                super::fixture::result_contract("ordinary.result.v1"),
                WorthServerProductOperationBasisKind::DurableProductDerived,
                WorthServerProductOperationSupportSnapshot::production_admitted("ordinary-only"),
            )
            .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
        )
}
pub(super) fn assert_refused(
    session: &WorthServerWorthNativeSession,
    lease: &ExecutionResourceLease<'_>,
) {
    let result = session
        .product_operations()
        .execute_shared_read_batch_with_lease(
            vec![WorthServerProductOperationInput::new(
                "ordinary.only",
                WorthServerProductOperationPayload::json(
                    "ordinary.payload.v1",
                    serde_json::json!({}),
                ),
            )
            .with_basis_digest("basis:r0")],
            lease,
        )
        .expect("funded packet reaches its adapter");
    assert_eq!(result.operations().len(), 1);
    let WorthServerProductOperationOutcome::Denied(denial) = result.operations()[0].outcome()
    else {
        panic!("unimplemented checked lane must deny")
    };
    assert_eq!(denial.reason_key(), "product.checked_execution_unsupported");
}
