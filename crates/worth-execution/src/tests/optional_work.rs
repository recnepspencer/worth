//! Absent ceilings preserve the original request's other limits and exact work.
use super::*;
use crate::{
    ExecutionRequest, ExecutionWorkCeiling, MapKernelFailure, MapKernelStop, MapOutcome,
    SerialRequest,
};

fn optional_request(work: Option<u64>) -> LeaseRequest {
    let mut admitted = request(1, 2_000, 1);
    admitted.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, work),
    );
    admitted
}

#[test]
fn absent_request_ceiling_preserves_serial_and_leased_charged_work() {
    let _guard = TEST_LOCK.lock().unwrap();
    let admitted = optional_request(None);
    let serial = SerialRequest::from_policy(&admitted.policy, admitted.cancellation.clone(), None);
    let lease = authority().request_lease(admitted).unwrap();
    for original in [
        ExecutionRequest::serial(&serial),
        ExecutionRequest::leased(&lease),
    ] {
        assert_eq!(original.work_ceiling(), None);
        let (outcome, report) = original
            .run(ExecutionWorkCeiling::new(None), |backing| {
                adversarial::map(&[2, 3], 0).run(backing, |value, context| {
                    context.checkpoint(4_500)?;
                    Ok::<_, MapKernelFailure<()>>(*value * 2)
                })
            })
            .unwrap();
        let MapOutcome::Complete { values, .. } = outcome else {
            panic!("absent work ceiling refused lawful work")
        };
        assert_eq!(values, vec![4, 6]);
        assert_eq!(report.charged_work(), 9_000);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn explicit_request_and_computation_ceilings_still_refuse_before_result() {
    let _guard = TEST_LOCK.lock().unwrap();
    for (request_work, computation_work) in [(Some(2), None), (None, Some(2))] {
        let admitted = optional_request(request_work);
        let serial =
            SerialRequest::from_policy(&admitted.policy, admitted.cancellation.clone(), None);
        let lease = authority().request_lease(admitted).unwrap();
        for original in [
            ExecutionRequest::serial(&serial),
            ExecutionRequest::leased(&lease),
        ] {
            let (outcome, _) = original
                .run(ExecutionWorkCeiling::new(computation_work), |backing| {
                    adversarial::map(&[1], 0).run(backing, |_, context| {
                        context.checkpoint(3)?;
                        Ok::<_, MapKernelFailure<()>>(1_u64)
                    })
                })
                .unwrap();
            assert!(
                matches!(outcome, MapOutcome::Stopped { completed_prefix, reason: crate::MapStop::WorkExhausted { .. }, .. } if completed_prefix.is_empty())
            );
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn absent_child_ceiling_cannot_widen_a_finite_parent() {
    let _guard = TEST_LOCK.lock().unwrap();
    let finite = authority()
        .request_lease(optional_request(Some(7)))
        .unwrap();
    assert!(matches!(
        finite.child(optional_request(None)),
        Err(LeaseDenial::WorkLimitExceedsParent)
    ));
    let unlimited = authority().request_lease(optional_request(None)).unwrap();
    assert_eq!(
        unlimited
            .child(optional_request(None))
            .unwrap()
            .policy()
            .budget()
            .work_ceiling(),
        None
    );
    assert_eq!(
        unlimited
            .child(optional_request(Some(7)))
            .unwrap()
            .policy()
            .budget()
            .work_ceiling(),
        Some(7)
    );
}

#[test]
fn absent_work_ceiling_preserves_original_interruption_before_execution() {
    let _guard = TEST_LOCK.lock().unwrap();
    for cancelled in [false, true] {
        let source = CancellationSource::new();
        let mut admitted = optional_request(None);
        admitted.cancellation = source.token();
        admitted.deadline = (!cancelled).then(Instant::now);
        let serial = SerialRequest::from_policy(
            &admitted.policy,
            admitted.cancellation.clone(),
            admitted.deadline,
        );
        let lease = authority().request_lease(admitted).unwrap();
        if cancelled {
            source.cancel();
        }
        for original in [
            ExecutionRequest::serial(&serial),
            ExecutionRequest::leased(&lease),
        ] {
            let expected = if cancelled {
                MapKernelStop::Cancelled
            } else {
                MapKernelStop::DeadlineElapsed
            };
            assert_eq!(original.check_live(), Err(expected));
            let contacts = AtomicBool::new(false);
            assert!(original
                .run(ExecutionWorkCeiling::new(None), |_| {
                    contacts.store(true, Ordering::SeqCst);
                })
                .is_err());
            assert!(!contacts.load(Ordering::SeqCst));
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn absent_work_ceiling_keeps_counter_overflow_typed() {
    let _guard = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(optional_request(None)).unwrap();
    let outcome = adversarial::map(&[1], 0).run(Some(&lease), |_, context| {
        context.checkpoint(u64::MAX)?;
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(1_u64)
    });
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: crate::MapStop::Failure {
                cause: MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow),
                ..
            },
            ..
        }
    ));
    assert_eq!(lease.charged_memory_bytes(), 0);
}
