//! Original controls stop bounded scalar construction and always stop publication.
use std::time::Duration;
use worth_execution::{
    CancellationSource, MonotonicTimeController, SerialMemoryBudget, SerialRequest,
};
use worth_execution::{
    ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial, MapKernelStop,
    WorkCeilingDenial,
};

#[test]
fn stopped_array_writes_refuse_within_the_control_chunk_and_cannot_seal() {
    for expected in [MapKernelStop::Cancelled, MapKernelStop::DeadlineElapsed] {
        for replacement in [false, true] {
            let time = MonotonicTimeController::new().unwrap();
            let now = time.hold();
            let cancellation = CancellationSource::new();
            let budget = SerialMemoryBudget::new(None);
            let original = SerialRequest::from_memory(
                budget.clone(),
                cancellation.token(),
                Some(time.clock().deadline_at(now + Duration::from_secs(1))),
            );
            let request = ExecutionRequest::serial(&original);
            let mut builder = ExecutionRequestArrayBuilder::allocate(request, 64).unwrap();
            builder.push(0u64).unwrap();
            if expected == MapKernelStop::Cancelled {
                cancellation.cancel();
            } else {
                time.advance(Duration::from_secs(2)).unwrap();
            }
            let mut refusal = None;
            // This is the declared control latency bound, not a cost golden:
            // at most 32 scalar moves can occur between control observations.
            for value in 1..=32 {
                let result = if replacement {
                    builder.replace_initialized(0, value)
                } else {
                    builder.push(value)
                };
                if let Err(cause) = result {
                    refusal = Some(cause);
                    break;
                }
            }
            assert_eq!(
                refusal,
                Some(ExecutionRequestArrayDenial::Request(
                    WorkCeilingDenial::Stopped(expected)
                ))
            );
            match builder.seal() {
                Err(ExecutionRequestArrayDenial::Request(WorkCeilingDenial::Stopped(cause))) => {
                    assert_eq!(cause, expected)
                }
                _ => panic!("a stopped partial array cannot publish"),
            }
            assert_eq!(budget.charged_memory_bytes(), 0);
        }
    }
}
