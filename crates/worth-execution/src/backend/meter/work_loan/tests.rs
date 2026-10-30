//! Real SDK scopes, descendants and worker dispatch own the work loan's authority.
use super::*;
use crate::{CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest};

#[test]
fn work_loan_charges_current_descendant_once_and_closes_at_scope_exit() {
    let storage = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&storage);
    let mut retained = None;
    let mut saved_limits = None;
    let (_, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&storage), |parent| {
            let loan = parent.work_loan(&execution).unwrap();
            loan.checkpoint(3).unwrap();
            let _ = execution
                .run_native_with_checkpoint(|child| {
                    assert_eq!(
                        loan.checkpoint_in(parent, 1),
                        Err(ExecutionWorkLoanDenial::ForeignScope)
                    );
                    loan.checkpoint(5).unwrap();
                    assert_eq!(child.charged_work(), 5);
                    let child_loan = child.work_loan(&execution).unwrap();
                    child_loan.checkpoint(2).unwrap();
                    loan.checkpoint(1).unwrap();
                    assert_eq!(child.charged_work(), 8);
                    saved_limits = Some(child.meter.borrow().limits.clone());
                })
                .unwrap();
            assert_eq!(parent.charged_work(), 11);
            assert_eq!(loan.checkpoint(1), Ok(()));
            {
                let _foreign = crate::backend::detach_meter_for_test();
                let _ = execution
                    .run_native_with_checkpoint(|_| {
                        assert_eq!(
                            loan.checkpoint(1),
                            Err(ExecutionWorkLoanDenial::ForeignScope)
                        );
                    })
                    .unwrap();
            }
            retained = Some(loan);
        })
        .unwrap();
    assert_eq!(report.charged_work(), 12);
    let retained = retained.unwrap();
    assert_eq!(
        retained.checkpoint(1),
        Err(ExecutionWorkLoanDenial::ClosedScope)
    );
    // Even an authentic saved descendant image cannot reopen its closed issuer.
    let mut saved = KernelContext::new(saved_limits.unwrap());
    let _active = saved.enter();
    assert_eq!(
        retained.checkpoint(1),
        Err(ExecutionWorkLoanDenial::ClosedScope)
    );
    assert!(saved.work_loan(&execution).is_err());
}

#[test]
fn work_loan_uses_genuine_framework_worker_lineage_and_original_limits() {
    let _serial = crate::tests::TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let lease = crate::tests::authority()
        .request_lease(crate::tests::request(4, 1_000_000, 100))
        .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let (_, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(Some(&lease), None, |context| {
            let loan = context.work_loan(&execution).unwrap();
            let outcome = crate::backend::run_checked_batch(
                Some(&lease),
                &crate::tests::batch(&[1, 1], 0),
                crate::backend::BackendKind::Native,
                &|_, context| {
                    loan.checkpoint(7).unwrap();
                    assert_eq!(context.charged_work(), 7);
                    Ok::<_, crate::MapKernelFailure<()>>(1_u64)
                },
            );
            assert!(outcome.stop.is_none());
        })
        .unwrap();
    assert_eq!(report.charged_work(), 14);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn work_loan_preserves_mixed_ceiling_overflow_and_parent_cancellation() {
    let parent = CancellationSource::new();
    let storage = SerialRequest::from_memory(SerialMemoryBudget::new(None), parent.token(), None);
    let child = storage
        .clone()
        .with_cancellation(CancellationSource::new().token());
    let execution = ExecutionRequest::serial(&child);
    let (_, report) = ExecutionWorkCeiling::new(Some(10))
        .run_with_checkpoint(None, Some(&storage), |context| {
            let loan = context.work_loan(&execution).unwrap();
            loan.checkpoint(7).unwrap();
            context.checkpoint(2).unwrap();
            assert_eq!(
                loan.checkpoint(2),
                Err(ExecutionWorkLoanDenial::Request(KernelStop::WorkCeiling))
            );
            assert!(loan.poll().is_err());
        })
        .unwrap();
    assert_eq!(
        report.charged_work(),
        9,
        "a denied visit executes no unbilled prefix"
    );

    let (_, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&storage), |context| {
            let loan = context.work_loan(&execution).unwrap();
            loan.checkpoint(u64::MAX).unwrap();
            assert_eq!(
                loan.checkpoint(1),
                Err(ExecutionWorkLoanDenial::Request(
                    KernelStop::WorkCounterOverflow
                ))
            );
        })
        .unwrap();
    assert_eq!(report.charged_work(), u64::MAX);

    let _ = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&storage), |context| {
            let loan = context.work_loan(&execution).unwrap();
            execution
                .run_native_with_checkpoint(|_| {
                    parent.cancel();
                    assert!(!child.cancellation.is_cancelled());
                    assert_eq!(
                        loan.checkpoint(1),
                        Err(ExecutionWorkLoanDenial::Request(KernelStop::Cancelled))
                    );
                })
                .unwrap_err();
        })
        .unwrap();
}
