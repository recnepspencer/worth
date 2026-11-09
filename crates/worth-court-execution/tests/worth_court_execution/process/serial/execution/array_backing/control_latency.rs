//! The physical builder observes an already-cancelled lease before moving a value.
use crate::array_backing::isolated_context::*;
use worth_execution::{
    CancellationSource, ExecutionAllocationDenialClass, ExecutionAllocationPolicy,
    ExecutionArrayBuilder, MapKernelStop, WorkCeilingDenial,
};

#[test]
fn stopped_array_writes_observe_cancellation_before_each_move() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }
    let _serial = crate::test_serialization::guard(&SERIAL);
    let payload = std::alloc::Layout::array::<u64>(64).unwrap().size() as u64;
    let source = CancellationSource::new();
    let mut declaration = request(payload);
    declaration.cancellation = source.token();
    let lease = authority().request_lease(declaration).unwrap();
    let mut builder =
        ExecutionArrayBuilder::allocate(64, ExecutionAllocationPolicy::Execution(&lease)).unwrap();
    builder.push(0u64).unwrap();
    source.cancel();
    let denied = builder.push(1).unwrap_err();
    assert_eq!(
        denied.classify(),
        ExecutionAllocationDenialClass::Request(WorkCeilingDenial::Stopped(
            MapKernelStop::Cancelled
        ))
    );
    assert_eq!(
        builder.elements(),
        &[0],
        "no move may occur after cancellation was observed"
    );
    assert!(builder.seal().is_err());
    assert_eq!(lease.charged_memory_bytes(), 0);
}
