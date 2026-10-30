#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_operation_request_runtime as operation_request_runtime;

pub(crate) use serde_json::json;
pub(crate) use {
    worth_server::request_context::DiagnosticRichnessProfile,
    worth_server::WorthServerCompatHttpRouteFamily, worth_server::WorthServerOperationFamily,
    worth_server::WorthServerOperationInputEnvelope,
    worth_server::WorthServerOperationRequestDenialCode,
    worth_server::WorthServerOperationRequestInput,
};

pub(crate) use {
    worth_testkit_server::shared_support_operation_request_runtime::compat_prepared_request,
    worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server,
    worth_testkit_server::shared_support_operation_request_runtime::worth_native_admission,
    worth_testkit_server::shared_support_operation_request_runtime::worth_native_resolved_context,
};

pub(crate) fn denied_code(
    result: Result<
        worth_server::WorthServerOperationRequest,
        worth_server::WorthServerOperationRequestDenial,
    >,
) -> WorthServerOperationRequestDenialCode {
    match result {
        Ok(value) => panic!("expected denied operation request, got {value:?}"),
        Err(denial) => denial.code(),
    }
}
