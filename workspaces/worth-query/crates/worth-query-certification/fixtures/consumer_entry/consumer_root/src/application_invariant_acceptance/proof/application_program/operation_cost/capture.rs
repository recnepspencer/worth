//! Actual typed-payload refusal returns the entire original prepared builder.
use super::*;
use worth_query_host::facade::primary_graph::{
    bound_advancement_requests_on_this_thread_for_test,
    place_managed_computations_on_this_thread_for_test, WorthQueryExecutionObservation,
    WorthQueryExecutionPlacementForTest,
};
use worth_query_host::facade::runtime::{
    ExecutionAllocationDenialKind, ExecutionAllocationPolicy, LeaseDenial, MemoryLimitLevel,
};
#[test]
fn ordinary_capture_refusal_returns_original_source_conditions_and_key_for_fresh_retry() {
    let runtime = installation::install(1, 0);
    let adapter = authentication::admit(runtime.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = runtime.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "focus-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let original = input("focus-a");
    reset_planar_source_adjustment_contacts(&original);
    let builder = request
        .mutate(Original(original))
        .expect_source(source)
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            BodyKey::reference::<Schema>(),
            ApplicationEncodedScalarValue::try_new("focus-a".to_owned()).unwrap(),
        ));
    let key = 223_001;
    let payload = std::alloc::Layout::array::<PlanarSourceAdjustment>(1)
        .unwrap()
        .size();
    assert!(payload > 1);
    // Existing scoped placement bounds the actual initiating request, not a fake allocator.
    let (builder, returned_key, denial) = {
        let _control = WorthQueryExecutionObservation::install_on_this_thread();
        place_managed_computations_on_this_thread_for_test(
            WorthQueryExecutionPlacementForTest::Serial,
        );
        let installed = resources::EXECUTION_POLICY.budget();
        bound_advancement_requests_on_this_thread_for_test(None);
        let (opening, available, declared) = runtime
            .runtime()
            .with_application_advancement(&scope, |phase| {
                let execution = phase
                    .checked_execution_request_for(runtime.runtime().product_runtime())
                    .unwrap();
                let ExecutionAllocationPolicy::Execution(parent) = execution.allocation_policy()
                else {
                    panic!("Query Serial must retain its real bounded execution lease")
                };
                (
                    parent.charged_memory_bytes(),
                    parent.available_memory_bytes(),
                    parent.policy().budget().charged_memory_bytes(),
                )
            })
            .unwrap();
        assert!(opening > 0, "opening holds actual framework resources");
        assert_eq!(opening.checked_add(available).unwrap(), declared);
        assert_eq!(declared, installed.charged_memory_bytes());
        let ceiling = opening
            .checked_add(u64::try_from(payload).unwrap())
            .unwrap()
            .checked_sub(1)
            .unwrap();
        assert!(ceiling > 0 && ceiling < declared);
        bound_advancement_requests_on_this_thread_for_test(Some(
            worth_foundational::ExecutionBudget::new(
                installed.max_workers(),
                ceiling,
                installed.work_ceiling(),
            ),
        ));
        match builder.idempotency_owned(key) {
            Err(refusal) => refusal,
            Ok(_) => panic!("typed payload exceeds the actual native request memory"),
        }
    };
    let WorthQueryOperationInputCaptureDenial::Allocation(ref allocation) = denial else {
        panic!("prerequisites must reach the real typed payload allocation: {denial:?}")
    };
    assert_eq!(
        allocation.requested_payload_bytes(),
        Some(u64::try_from(payload).unwrap())
    );
    let ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(limit)) =
        allocation.kind()
    else {
        panic!("actual typed payload must meet its native lease refusal: {allocation:?}")
    };
    assert_eq!(limit.requested, u64::try_from(payload).unwrap());
    assert_eq!(limit.admitted, u64::try_from(payload - 1).unwrap());
    assert_eq!(limit.level, MemoryLimitLevel::Policy { ancestor: 1 });
    assert_eq!(returned_key, key);
    assert_eq!(planar_source_adjustment_contacts(&input("focus-a")), (0, 0));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    let retained = builder.into_capture_refusal(returned_key, denial);
    drop(principal);
    drop(scope);
    let foreign = installation::install(1, 0);
    let foreign_adapter = authentication::admit(foreign.installed_schema());
    let foreign_scope = authentication::request_scope();
    let foreign_actor = authentication::block_on(foreign_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &foreign_scope,
    ))
    .unwrap();
    let (retained, refusal) = retained
        .retry_capture(&foreign.request(&foreign_actor, &foreign_scope))
        .err()
        .expect("foreign receiving runtime cannot capture the original branch");
    assert!(matches!(
        refusal,
        WorthQueryOperationInputCaptureRetryDenial::FreshRequestMismatch
    ));
    assert_eq!(planar_source_adjustment_contacts(&input("focus-a")), (0, 0));
    // Fresh authority, same original owner/meaning; no initiating request remains.
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = runtime.request(&principal, &scope);
    let description = retained
        .retry_capture(&request)
        .unwrap_or_else(|(_, denial)| panic!("fresh original capture: {denial:?}"));
    let owner = description
        .retain_operation::<Program, Action>(&runtime)
        .unwrap_or_else(|(_, denial)| panic!("original exact descriptor: {denial:?}"));
    let settled = settle(owner, &request);
    assert_eq!(planar_source_adjustment_contacts(&input("focus-a")), (1, 1));
    assert_eq!(settled.handler_result().unwrap().changed_vertices, 1);
    assert_eq!(journey::values(&request, 1), vec![2]);
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("lawful close: {denial:?}"));
}
