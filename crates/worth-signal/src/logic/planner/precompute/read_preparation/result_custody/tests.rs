use super::*;
use crate::data::aspect::AspectVersion;
use crate::data::output::NodeEvaluationResult;
use worth_execution::{
    CancellationToken, ExecutionRequest, MemoryLimitDenial, SerialMemoryBudget, SerialRequest,
};

fn output() -> PreparedEvaluation {
    let mut prepared =
        PreparedEvaluation::from_result(NodeEvaluationResult::from_version(AspectVersion::zero()));
    prepared
        .trace_data
        .labels
        .push("original-output".repeat(32768));
    prepared
}

fn handoff(limit: Option<u64>) -> (u64, u64) {
    let parent = SerialMemoryBudget::new(limit);
    let serial = SerialRequest::from_memory(parent.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let observed = request
        .in_scope(|lease| {
            let mut preparation = SignalPreparationBudget::for_request(limit, lease)?;
            preparation.claim_vec::<OutputSlot>(1)?;
            let mut memory = parent.reserve(0).unwrap();
            let measured = request.run_measured_result(
                worth_execution::ExecutionWorkCeiling::new(None),
                0,
                &mut memory,
                |_, _| Ok::<_, MapKernelFailure<SignalError>>(output()),
            );
            let peak = measured.report.physical().peak_charged_memory_bytes();
            let value = measured.result.expect("one real measured result fits");
            let original_backing = value.trace_data.labels[0].as_ptr();
            let heap = memory.bytes();
            let slots = vec![Mutex::new(Some(Ok(ReservedOutput {
                outcome: Ok(value),
                memory,
            })))];
            let moved = collect_outputs(slots, 1, &mut preparation)?;
            assert_eq!(
                moved.values[0].trace_data.labels[0].as_ptr(),
                original_backing
            );
            assert_eq!(moved.memory.len(), 1);
            assert_eq!(moved.memory[0].bytes(), heap);
            Ok::<_, SignalError>((moved, preparation, peak, heap))
        })
        .unwrap()
        .unwrap();
    let (moved, preparation, peak, heap) = observed;
    if limit.is_some() {
        // A second payload would be a real allocation, unlike the move.
        assert!(
            matches!(parent.reserve(heap), Err(MemoryLimitDenial::Capacity {
            requested, admitted, ..
        }) if requested == heap && admitted < heap)
        );
    }
    drop((moved, preparation));
    assert_eq!(parent.charged_memory_bytes(), 0);
    (peak, heap)
}

#[test]
fn finite_parent_funds_one_original_payload_through_move_handoff() {
    let (framework_and_payload, heap) = handoff(None);
    let vector_backings = (std::mem::size_of::<OutputSlot>()
        + std::mem::size_of::<PreparedEvaluation>()
        + std::mem::size_of::<ExecutionMemoryReservation>()) as u64;
    let limit = framework_and_payload.checked_add(vector_backings).unwrap();
    assert!(
        limit < heap.checked_mul(2).unwrap(),
        "the finite parent cannot fund two payloads"
    );
    let (_, finite_heap) = handoff(Some(limit));
    assert_eq!(finite_heap, heap);
}
