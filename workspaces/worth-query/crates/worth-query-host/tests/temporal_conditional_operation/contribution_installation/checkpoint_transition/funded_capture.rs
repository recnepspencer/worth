//! A caller lease funds an Execution-mode capture as the call's one budget.
use super::*;
use application_installation::WorthQueryCheckpointCaptureDenial as CaptureDenial;
use original_request::{request, transition_original};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::runtime::{
    CancellationToken, ExecutionAllocationDenialKind as AllocationKind, LeaseDenial,
};

fn funded_target() -> original_request::Target {
    let (source, predecessor) = source();
    let setup = primary_graph::query_test_execution_authority_for_test()
        .request_lease(request(CancellationToken::new()))
        .unwrap();
    transition_original(
        ExecutionRequest::leased(&setup),
        source,
        predecessor,
        |writer, _| admission::author_record(writer, "funded-capture", 151),
    )
    .unwrap()
}

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
    // The call's temporaries are returned; only the retained frame stays charged.
    assert_eq!(caller.charged_memory_bytes(), charge);
    let system = target
        .with_host_advancement(|phase| {
            target.capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
        })
        .unwrap()
        .unwrap();
    assert_eq!(system.bytes(), checkpoint.bytes());
    assert_eq!(system.charged_payload_bytes(), None);
    drop(target);
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
        panic!("the unrelated lease is the exact typed cause: {refusal:?}");
    };
    assert_eq!(
        refusal.kind(),
        AllocationKind::Lease(LeaseDenial::UnrelatedNestedLease)
    );
    assert_eq!(foreign.charged_memory_bytes(), 0);
    assert_eq!(caller.charged_memory_bytes(), 0);
}
