use super::*;
use crate::{
    backend::{run_scope, scope::run_scope_within},
    tests::{authority, request, TEST_LOCK},
    ExecutionResourceLease,
};

fn kernel(context: &mut KernelContext<'_, '_>) -> Result<u64, KernelFailure<()>> {
    for units in [2, 0, 3, 2] {
        context.checkpoint(units)?;
    }
    Ok(73)
}

fn receipt(lease: &ExecutionResourceLease<'_>) -> ExecutionKernelReceipt {
    let storage = ExecutionKernelReceiptStorage::new(lease.reserve_memory(0).unwrap()).unwrap();
    run_scope(Some(lease), 0, 512, |context| {
        let (result, receipt) = context.record_kernel(&storage, kernel);
        assert_eq!(result.unwrap(), 73);
        Ok::<_, KernelFailure<()>>(receipt.unwrap())
    })
    .result
    .unwrap()
}

#[test]
fn native_receipt_matches_fresh_work_span_and_every_tight_checkpoint_prefix() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let receipt = receipt(&lease);
    assert_eq!(receipt.charged_work(), 7);
    for ceiling in 0..=9 {
        let fresh = run_scope_within(Some(&lease), 0, 0, true, ceiling, None, kernel);
        let carried = run_scope_within(Some(&lease), 0, 0, true, ceiling, None, |context| {
            context.carry_kernel(&receipt)?;
            Ok::<_, KernelFailure<()>>(73_u64)
        });
        assert_eq!(
            format!("{:?}", fresh.result),
            format!("{:?}", carried.result),
            "ceiling={ceiling}"
        );
        assert_eq!(
            fresh.report.charged_work(),
            carried.report.charged_work(),
            "ceiling={ceiling}"
        );
        assert_eq!(
            fresh.report.charged_span(),
            carried.report.charged_span(),
            "ceiling={ceiling}"
        );
    }
}

#[test]
fn context_questions_and_handled_nested_failures_do_not_issue_a_receipt() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let storage = ExecutionKernelReceiptStorage::new(lease.reserve_memory(0).unwrap()).unwrap();
    let question = run_scope(Some(&lease), 0, 0, |context| {
        let (result, receipt) = context.record_kernel(&storage, |context| {
            Ok::<_, KernelFailure<()>>(context.remaining_work())
        });
        assert!(receipt.is_none());
        assert_eq!(result.unwrap(), 100);
        let (result, receipt) = context.record_kernel(&storage, |context| {
            Ok::<_, KernelFailure<()>>(context.take_nested_computation_stop())
        });
        assert_eq!(result.unwrap(), None);
        assert!(
            receipt.is_none(),
            "even an absent nested cause was observed"
        );
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(question.result.is_ok());
    let nested = run_scope(Some(&lease), 0, 0, |context| {
        let (result, receipt) = context.record_kernel(&storage, |context| {
            let child = run_scope(Some(&lease), 0, 0, |child| {
                child.checkpoint(1)?;
                Err::<(), _>(KernelFailure::Domain(()))
            });
            assert!(child.result.is_err());
            assert_eq!(
                context.take_nested_computation_stop(),
                Some(super::super::NestedComputationStop::DomainFailure)
            );
            context.checkpoint(2)?;
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(receipt.is_none());
        result
    });
    assert!(nested.result.is_ok(), "{:?}", nested.result);
    assert_eq!(
        (nested.report.charged_work(), nested.report.charged_span()),
        (3, 3)
    );
}

#[test]
fn swallowed_cancellation_never_issues_a_receipt_and_stops_real_carry() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let receipt = receipt(&lease);
    let cancellation = crate::CancellationSource::new();
    let mut requested = request(1, 2_000, 100);
    requested.cancellation = cancellation.token();
    let interrupted = authority().request_lease(requested).unwrap();
    let storage =
        ExecutionKernelReceiptStorage::new(interrupted.reserve_memory(0).unwrap()).unwrap();
    let outcome = run_scope(Some(&interrupted), 0, 0, |context| {
        let (_, captured) = context.record_kernel(&storage, |context| {
            cancellation.cancel();
            assert_eq!(context.carry_kernel(&receipt), Err(KernelStop::Cancelled));
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(captured.is_none());
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        outcome.result,
        Err(crate::backend::scope::ScopeStop::Failure(
            KernelFailure::Stop(KernelStop::Cancelled)
        ))
    ));
}

#[test]
fn cloned_receipts_keep_funding_after_storage_and_original_drop() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let original = receipt(&lease);
    let cloned = original.clone();
    drop(original);
    assert!(lease.reserve_memory(2_000).is_err());
    assert_eq!(cloned.charged_work(), 7);
    drop(cloned);
    assert!(lease.reserve_memory(2_000).is_ok());
}

#[test]
fn carried_cost_and_prefix_observation_cannot_be_recaptured_as_owner_checkpoints() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let prior = receipt(&lease);
    let storage = ExecutionKernelReceiptStorage::new(lease.reserve_memory(0).unwrap()).unwrap();
    let outcome = run_scope(Some(&lease), 0, 0, |context| {
        let (result, captured) = context.record_kernel(&storage, |context| {
            context
                .carry_kernel(&prior)
                .map_err(KernelFailure::<()>::from)
        });
        result?;
        assert!(captured.is_none());
        let (result, captured) = context.record_kernel(&storage, |context| {
            Ok::<_, KernelFailure<()>>(context.charged_cost())
        });
        assert_eq!(result.unwrap(), (7, 7));
        assert!(captured.is_none());
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(outcome.result.is_ok());
    assert_eq!(outcome.report.charged_work(), 7);
}

#[test]
fn a_panicking_capture_releases_backing_before_its_admission() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let storage = ExecutionKernelReceiptStorage::new(lease.reserve_memory(0).unwrap()).unwrap();
    let outcome = run_scope(Some(&lease), 0, 0, |context| {
        context
            .record_kernel(&storage, |context| -> Result<(), KernelFailure<()>> {
                context.checkpoint(2)?;
                panic!("actual captured kernel panic");
            })
            .0
    });
    assert!(matches!(
        outcome.result,
        Err(crate::backend::scope::ScopeStop::Failure(
            KernelFailure::Panic
        ))
    ));
    drop(storage);
    assert!(lease.reserve_memory(2_000).is_ok());
}
