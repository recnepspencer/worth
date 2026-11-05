//! The complete embedded checkpoint, including its outer bytes, keeps custody.
use super::{fixture::Grid, original, recovery::publish_grid, transaction::fixture};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Barrier,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::*, runtime::RelationalRuntime, transactions::RelationalExecutionDenialCause,
};

fn populated(samples: u32) -> (RelationalRuntime, usize) {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish_grid(
        &runtime,
        &execution,
        &contract,
        &Grid {
            rows: 1,
            columns: samples,
            samples: vec![0; samples as usize],
        },
    );
    (runtime, 43 + samples as usize * 8)
}

#[test]
fn content_whole_checkpoint_original_custody_survives_clones_until_last_release() {
    let (runtime, frame_length) = populated(65_536);
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let retained = budget.charged_memory_bytes();
    assert!(
        retained >= (checkpoint.bytes().len() + frame_length) as u64,
        "outer checkpoint bytes and member bytes each own their actual grant"
    );
    let alias = checkpoint.clone();
    drop(checkpoint);
    assert_eq!(budget.charged_memory_bytes(), retained);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn content_whole_checkpoint_peak_budget_refuses_before_allocating_the_outer_frame() {
    let (runtime, frame_length) = populated(65_536);
    let cancel = CancellationSource::new();
    let probe = original(&cancel);
    let probe = ExecutionRequest::serial(&probe);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&probe, probe.allocation_policy())
        .unwrap();
    let outer = checkpoint.bytes().len() as u64;
    drop(checkpoint);
    let budget = SerialMemoryBudget::new(Some(outer + frame_length as u64 - 1));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let refusal = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap_err();
    assert!(
        matches!(refusal, RelationalNativeCheckpointCaptureDenial::Request(RelationalExecutionDenialCause::PolicyMemoryExhausted { requested, admitted, .. }) if requested == outer && admitted < outer),
        "{refusal:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}

#[test]
fn content_whole_checkpoint_cancellation_after_outer_allocation_releases_all_capture_bytes() {
    let (runtime, frame_length) = populated(1_048_576);
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let done = AtomicBool::new(false);
    let ready = Barrier::new(2);
    let (refusal, observed) = std::thread::scope(|scope| {
        let observer = scope.spawn(|| {
            ready.wait();
            while !done.load(Ordering::Acquire) {
                // The Native member and intermediate envelope together are
                // roughly two frames. This threshold requires the outer copy.
                if budget.charged_memory_bytes() > 3 * frame_length as u64 {
                    cancel.cancel();
                    return true;
                }
                std::thread::yield_now();
            }
            false
        });
        ready.wait();
        let result = runtime
            .durability_authority()
            .native_checkpoint(&execution, execution.allocation_policy());
        done.store(true, Ordering::Release);
        (result.unwrap_err(), observer.join().unwrap())
    });
    assert!(observed);
    assert!(
        matches!(
            refusal,
            RelationalNativeCheckpointCaptureDenial::Request(
                RelationalExecutionDenialCause::Cancelled
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}
