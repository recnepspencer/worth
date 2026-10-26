use super::*;
use crate::data::aspect::AspectVersion;
use crate::data::output::NodeEvaluationResult;
use worth_execution::{
    CancellationToken, ExecutionRequest, MemoryLimitDenial, SerialMemoryBudget, SerialRequest,
};

#[test]
fn physical_allocation_stop_never_reads_a_domain_result_slot() {
    let kind = worth_execution::ExecutionPhysicalAllocationDenialKind::Allocator;
    let restored = restore_stop(MapStop::Allocation(kind), &[], 0);
    assert!(matches!(restored, MapStop::Allocation(actual) if actual == kind));
}

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
            let computed = allocate_outputs(1, &mut preparation)?;
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
            let slots = vec![Mutex::new(Some(ReservedOutput {
                outcome: Ok(value),
                memory,
            }))];
            let moved = collect_outputs(slots, computed);
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

struct SaturatingCallback<'a> {
    parent: &'a SerialMemoryBudget,
    competitor: Mutex<Option<ExecutionMemoryReservation>>,
}

impl SignalPrecompute for SaturatingCallback<'_> {
    fn prepare(
        &self,
        _: crate::data::handle::NodeId,
        _: &crate::logic::prepared::ExecutionReadView<'_>,
        _: &mut MapKernelContext<'_, '_>,
        _: Option<super::super::super::callback::CheckedKernelCapacity>,
    ) -> Result<PreparedEvaluation, SignalError> {
        let remaining = self.parent.limit().unwrap() - self.parent.charged_memory_bytes();
        *self.competitor.lock().unwrap() = Some(self.parent.reserve(remaining).unwrap());
        Ok(output())
    }

    fn allows_bounded_inputs(&self) -> bool {
        true
    }
}

#[test]
fn result_custody_settles_without_late_admission_and_types_an_underquote() {
    let mut graph = SignalGraph::new();
    let node = graph.create_node();
    let snapshot = ExecutionSnapshot::new(&graph);
    let view = snapshot.read_view(node);
    for bound in [1, 1 << 20] {
        let parent = SerialMemoryBudget::new(Some(4 << 20));
        let serial = SerialRequest::from_memory(parent.clone(), CancellationToken::new(), None);
        let request = ExecutionRequest::serial(&serial);
        let callback = SaturatingCallback {
            parent: &parent,
            competitor: Mutex::new(None),
        };
        let memory = parent.reserve(bound).unwrap();
        let (captured, _) = request
            .run_native_with_checkpoint(|work| {
                compute_output(
                    node,
                    &view,
                    &callback,
                    super::super::super::callback::CheckedKernelCapacity {
                        scratch_bytes: Some(bound),
                        result_bytes: Some(bound),
                    },
                    work,
                    memory,
                )
            })
            .unwrap();
        assert!(callback.competitor.lock().unwrap().is_some());
        if bound == 1 {
            assert!(matches!(
                captured.outcome,
                Err(MapKernelFailure::ResultCapacityExceeded)
            ));
        } else {
            let value = captured.outcome.as_ref().unwrap();
            let actual = value
                .retained_heap_charge(&mut RetainedStoragePreparation::new(None))
                .unwrap()
                .bytes();
            assert!(actual > 0 && actual < bound);
            assert_eq!(captured.memory.bytes(), actual);
        }
        drop(captured);
        drop(callback);
        assert_eq!(parent.charged_memory_bytes(), 0);
    }
}
