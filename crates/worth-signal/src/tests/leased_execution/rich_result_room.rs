//! An adaptive grant must fund useful native heap before callback effects.
use super::support::private_authority_process::{
    run_with_private_authority, NativeExecutionEvidence,
};
use super::support::{authority, request};
use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, ChangedRegion, EvaluationOutput,
    EvaluationRequestMode, IntoEvaluationOutput, NodeContract, NodeEvaluationResult,
    PartitionSubscription, ScopePath, SignalError, SignalExecutionFailure,
    SignalExecutionStopReason, SignalGraph, SignalLeaseDenial,
};
use std::sync::atomic::{AtomicUsize, Ordering};

const VALUE: Aspect = Aspect::new(0);

fn result() -> NodeEvaluationResult {
    let mut result = NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, 7));
    // Exactly one slot, not a Vec push's spare-capacity growth.
    result.changed_aspect_regions =
        vec![(VALUE, ChangedRegion::exact(ScopePath::one("x").unwrap()))];
    result
}

fn attempt<O: IntoEvaluationOutput>(workers: usize, room: u64, make: fn() -> O) -> bool {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(
            NodeContract::wildcard()
                .with_produces(VALUE)
                .with_bounded_inputs(BoundedSignalInputs::default()),
        )
        .partitioned_output()
        .build();
    let sibling = authority()
        .request_lease(request(workers, u64::MAX))
        .unwrap();
    let lease = authority()
        .request_lease(request(workers, 10_000_000))
        .unwrap();
    let process_room = sibling.available_memory_bytes().unwrap();
    let hold = sibling.reserve_memory(process_room - room).unwrap();
    assert_eq!(lease.available_memory_bytes(), Some(room));
    let calls = AtomicUsize::new(0);
    let outcome = graph.evaluate_checked(
        &[node],
        EvaluationRequestMode::Default,
        &(),
        &|context| {
            calls.fetch_add(1, Ordering::SeqCst);
            assert!(
                context.result_capacity_bytes().unwrap()
                    >= NodeEvaluationResult::minimum_useful_heap_bytes()
            );
            Ok(make())
        },
        worth_execution::ExecutionRequest::leased(&lease),
    );
    let published = match outcome {
        Ok(report) => {
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(report.tasks_executed, 1);
            assert!(
                report
                    .execution
                    .last()
                    .unwrap()
                    .physical()
                    .peak_charged_memory_bytes()
                    <= room
            );
            assert_eq!(
                graph
                    .node_version_for_scope(
                        node,
                        VALUE,
                        Some(&PartitionSubscription::exact(ScopePath::one("x").unwrap()))
                    )
                    .unwrap(),
                7
            );
            true
        }
        Err(error) => {
            assert_eq!(
                calls.load(Ordering::SeqCst),
                0,
                "callback ran then failed at workers={workers}, room={room}: {error:?}"
            );
            assert!(memory_refusal(&error), "unexpected stop: {error:?}");
            assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 0);
            false
        }
    };
    assert_eq!(lease.charged_memory_bytes(), 0);
    drop(hold);
    assert_eq!(sibling.charged_memory_bytes(), 0);
    published
}

fn memory_refusal(error: &SignalError) -> bool {
    match error {
        SignalError::PreparationMemoryExhausted { .. }
        | SignalError::ExecutionAdmissionDenied(SignalLeaseDenial::MemoryExhausted(_)) => true,
        SignalError::ExecutionStopped(stop) => {
            assert_eq!(stop.publication_progress().completed_tasks(), 0);
            match stop.reason() {
                SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                | SignalExecutionStopReason::Admission(SignalLeaseDenial::MemoryExhausted(_)) => {
                    true
                }
                SignalExecutionStopReason::Failure {
                    cause: SignalExecutionFailure::Domain(nested),
                    ..
                } => memory_refusal(nested),
                _ => false,
            }
        }
        _ => false,
    }
}

fn sweep<O: IntoEvaluationOutput>(make: fn() -> O) -> usize {
    let mut samples = 0;
    for workers in [1, 2, 4] {
        let mut denied = 16 << 10;
        let mut published = 32 << 20;
        assert!(!attempt(workers, denied, make));
        assert!(attempt(workers, published, make));
        while denied + 1 < published {
            let middle = denied + (published - denied) / 2;
            if attempt(workers, middle, make) {
                published = middle;
            } else {
                denied = middle;
            }
        }
        let half_band = 300 << 10;
        let mut early = 0;
        let mut completed = 0;
        for room in (published.saturating_sub(half_band)..=published + half_band).step_by(256) {
            if attempt(workers, room, make) {
                completed += 1;
            } else {
                early += 1;
            }
            samples += 1;
        }
        assert!(early > 0 && completed > 0, "sweep must straddle admission");
    }
    samples
}

#[test]
fn native_heap_outputs_refuse_before_effects_across_the_admission_band() {
    if run_with_private_authority(
        "tests::leased_execution::rich_result_room::native_heap_outputs_refuse_before_effects_across_the_admission_band",
        NativeExecutionEvidence::RichResultRoom,
    ) { return; }
    // Independent storage measurement checks the actual native shape, including
    // all String and Vec capacities, against the type-derived useful minimum.
    let measured = result()
        .retained_heap_charge(&mut RetainedStoragePreparation::new(None))
        .unwrap()
        .bytes();
    assert_eq!(measured, NodeEvaluationResult::minimum_useful_heap_bytes());
    let direct = sweep(result);
    let detailed = sweep(|| EvaluationOutput::from_result(result()));
    println!("RICH_RESULT_ROOM {direct} direct and {detailed} detailed fine-step samples");
}
