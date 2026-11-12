pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::checkpoint_transition::funded_capture::*;

#[test]
fn execution_capture_charges_its_frame_to_the_caller_lease_end_to_end() {
    let target = funded_target();
    let caller = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    assert_eq!(caller.charged_memory_bytes(), 0);
    let checkpoint = target
        .with_funded_host_advancement(CapturePolicy::Execution(&caller), |phase| {
            target.capture_application_checkpoint(&phase, CapturePolicy::Execution(&caller))
        })
        .expect("the caller lease opens the host call it funds")
        .expect("the caller lease is in the capture worker's lineage");
    let charge = u64::try_from(checkpoint.bytes().len()).unwrap();
    assert_eq!(checkpoint.charged_payload_bytes(), Some(charge));
    // The frame and its actual immutable Native/World proof retain custody.
    let retained = caller.charged_memory_bytes();
    assert!(retained > charge);
    assert!(checkpoint.capture_origin().is_some());
    let shared = checkpoint.clone();
    assert_eq!(caller.charged_memory_bytes(), retained);
    drop(shared);
    assert_eq!(caller.charged_memory_bytes(), retained);
    let system = target
        .with_host_advancement(|phase| {
            target.capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
        })
        .unwrap()
        .unwrap();
    assert_eq!(system.charged_payload_bytes(), None);
    target
        .with_host_advancement(|phase| {
            let funded = target
                .runtime()
                .inspect_application_checkpoint(&phase, checkpoint.bytes())
                .unwrap();
            let ordinary = target
                .runtime()
                .inspect_application_checkpoint(&phase, system.bytes())
                .unwrap();
            assert_eq!(funded.model, ordinary.model);
        })
        .unwrap();
    drop(target);
    admission::assert_target_record(system, 151);
    assert_eq!(caller.charged_memory_bytes(), retained);
    admission::assert_target_record(checkpoint, 151);
    assert_eq!(caller.charged_memory_bytes(), 0);
}

#[test]
fn execution_capture_through_a_foreign_lease_still_refuses_unrelated_nesting() {
    let target = funded_target();
    let caller = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let foreign = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    let refusal = target
        .with_funded_host_advancement(CapturePolicy::Execution(&caller), |phase| {
            target.capture_application_checkpoint(&phase, CapturePolicy::Execution(&foreign))
        })
        .expect("the caller lease opens the host call it funds")
        .expect_err("a lease outside the worker lineage cannot fund the capture");
    let CaptureDenial::Allocation(refusal) = refusal else {
        panic!("the selected capture lease retains its exact admission cause: {refusal:?}");
    };
    assert_eq!(refusal.requested_payload_bytes(), None);
    assert_eq!(
        refusal.classify(),
        worth_execution::ExecutionAllocationDenialClass::Request(
            worth_execution::WorkCeilingDenial::Admission(
                worth_execution::LeaseDenial::UnrelatedNestedLease
            )
        )
    );
    assert_eq!(foreign.charged_memory_bytes(), 0);
    assert_eq!(caller.charged_memory_bytes(), 0);
}
