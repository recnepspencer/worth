pub(crate) use {
    worth_server::WorthServerCompatibilityMutationPreconditionContext,
    worth_server::WorthServerOperationReadinessDenialCode,
};

#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_operation_request_runtime as operation_request_runtime;

pub(crate) use {
    worth_testkit_server::shared_support_operation_request_runtime::compat_prepared_request,
    worth_testkit_server::shared_support_operation_request_runtime::compat_request_input,
    worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server,
};

pub(crate) fn prepared_request(
    server: &worth_server::WorthServer,
    builder: worth_server::WorthServerCompatibilityRequestInputBuilder,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
        builder
            .build()
            .expect("compatibility request should validate structurally"),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected prepared compatibility request, got {other:?}"),
    }
}
