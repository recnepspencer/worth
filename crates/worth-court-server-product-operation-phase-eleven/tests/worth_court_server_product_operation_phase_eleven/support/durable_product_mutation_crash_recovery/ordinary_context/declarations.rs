#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

pub(crate) use {serde_json::json, serde_json::Value};
pub(crate) use {
    worth_server::WorthServerCompletedProductOperation,
    worth_server::WorthServerDurableProductMutationConclusion,
    worth_server::WorthServerOperationAuthorizationPolicy,
    worth_server::WorthServerProductOperationOutcome,
    worth_server::WorthServerProductOperationSurfaceDenialCode,
};

#[allow(unused_imports)]
pub(crate) use worth_testkit_server::shared_support_durable_product_mutation as durable_support;

pub(crate) use {
    worth_testkit_server::shared_support_durable_product_mutation::driver::execute,
    worth_testkit_server::shared_support_durable_product_mutation::executor::DurableMutationCrashPoint,
    worth_testkit_server::shared_support_durable_product_mutation::executor::TestDurableProductExecutor,
    worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server,
    worth_testkit_server::shared_support_durable_product_mutation::runtime::build_server_with_mutation_policy,
    worth_testkit_server::shared_support_durable_product_mutation::runtime::prepared_mutation_request,
    worth_testkit_server::shared_support_durable_product_mutation::runtime::session,
    worth_testkit_server::shared_support_durable_product_mutation::runtime::session_with_principal,
};

pub(crate) fn crash_payload(crash: DurableMutationCrashPoint) -> Value {
    json!({
        "connection_id": "host-crash",
        "crash_point": crash.as_str(),
    })
}

pub(crate) fn assert_failed_at(
    operation: &WorthServerCompletedProductOperation,
    crash: DurableMutationCrashPoint,
) {
    match operation.outcome() {
        worth_server::WorthServerProductOperationOutcome::Failed(failure) => {
            assert_eq!(
                failure.reason_key(),
                format!("injected_crash_{}", crash.as_str())
            );
        }
        other => panic!("expected injected crash failure, got {other:?}"),
    }
}
