//! Actual embedded ingress must fund bytes before copying and release on refusal.
use super::{fixture::Grid, original, recovery::publish_grid, transaction::fixture};
use std::sync::atomic::{AtomicBool, Ordering};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{durability::*, transactions::RelationalExecutionDenialCause};

#[test]
fn content_inline_checkpoint_insufficient_original_memory_never_selects_a_root() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let grid = Grid {
        rows: 1,
        columns: 65_536,
        samples: vec![0; 65_536],
    };
    publish_grid(&runtime, &execution, &contract, &grid);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(524_330));
    let limited = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let limited = ExecutionRequest::serial(&limited);
    let (mut target, _) = fixture();
    let before = target
        .observe_branch(&target.main_branch_identity())
        .unwrap()
        .1;
    let refusal = target
        .durability_recovery()
        .restore_native_checkpoint(&limited, &checkpoint)
        .unwrap_err();
    assert!(
        matches!(
            refusal,
            RelationalRecoveryStop::Request(
                RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}

#[test]
fn content_inline_checkpoint_cancellation_after_backing_admission_releases_original_custody() {
    let producer_cancel = CancellationSource::new();
    let producer = original(&producer_cancel);
    let execution = ExecutionRequest::serial(&producer);
    let (runtime, contract) = fixture();
    let samples = 1_048_576;
    let grid = Grid {
        rows: 1,
        columns: samples,
        samples: vec![0; samples as usize],
    };
    publish_grid(&runtime, &execution, &contract, &grid);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(&execution, execution.allocation_policy())
        .unwrap();
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let receiving = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let receiving = ExecutionRequest::serial(&receiving);
    let done = AtomicBool::new(false);
    let ready = std::sync::Barrier::new(2);
    let (mut target, _) = fixture();
    let before = target
        .observe_branch(&target.main_branch_identity())
        .unwrap()
        .1;
    let (refusal, observed_backing) = std::thread::scope(|scope| {
        let observer = scope.spawn(|| {
            ready.wait();
            while !done.load(Ordering::Acquire) {
                // This independent memory observation cannot be reached by the
                // tiny manifest members. The actual content allocation exists.
                if budget.charged_memory_bytes() >= 8 * 1024 * 1024 {
                    cancel.cancel();
                    return true;
                }
                std::thread::yield_now();
            }
            false
        });
        ready.wait();
        let result = target
            .durability_recovery()
            .restore_native_checkpoint(&receiving, &checkpoint);
        done.store(true, Ordering::Release);
        (result.unwrap_err(), observer.join().unwrap())
    });
    assert!(observed_backing);
    assert!(
        matches!(
            refusal,
            RelationalRecoveryStop::Request(RelationalExecutionDenialCause::Cancelled)
        ),
        "{refusal:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}
