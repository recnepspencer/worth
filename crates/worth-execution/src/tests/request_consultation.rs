use super::*;
use crate::{ExecutionRequest, SerialRequest, WorkCeilingDenial};

#[test]
fn consultation_reads_both_backings() {
    let _serial = TEST_LOCK.lock().unwrap();
    let source = CancellationSource::new();
    let policy = request(1, 2_000, 3).policy;
    let serial = SerialRequest::from_policy(&policy, source.token(), None);
    let lease = authority()
        .request_lease(LeaseRequest {
            policy,
            cancellation: source.token(),
            deadline: None,
        })
        .unwrap();
    for execution in [
        ExecutionRequest::serial(&serial),
        ExecutionRequest::leased(&lease),
    ] {
        assert_eq!(execution.consult(), Ok(()));
    }
    source.cancel();
    for execution in [
        ExecutionRequest::serial(&serial),
        ExecutionRequest::leased(&lease),
    ] {
        assert_eq!(
            execution.consult(),
            Err(WorkCeilingDenial::Stopped(KernelStop::Cancelled))
        );
    }
    let expired = Instant::now() - Duration::from_secs(1);
    let serial = SerialRequest::from_policy(&policy, CancellationToken::new(), Some(expired));
    let lease = authority()
        .request_lease(LeaseRequest {
            policy,
            cancellation: CancellationToken::new(),
            deadline: Some(expired),
        })
        .unwrap();
    for execution in [
        ExecutionRequest::serial(&serial),
        ExecutionRequest::leased(&lease),
    ] {
        assert_eq!(
            execution.consult(),
            Err(WorkCeilingDenial::Stopped(KernelStop::DeadlineElapsed))
        );
    }
}

fn check_work_consultation(serial_backing: bool, refuse_charge: bool) {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 3)).unwrap();
    let serial = SerialRequest::from_policy(lease.policy(), CancellationToken::new(), None);
    let (execution, backing) = if serial_backing {
        (ExecutionRequest::serial(&serial), None)
    } else {
        (ExecutionRequest::leased(&lease), Some(&lease))
    };
    let outcome = crate::backend::run_scope_within::<(), (), _>(
        backing,
        0,
        0,
        true,
        3,
        backing.is_none().then_some(&serial),
        |work| {
            work.checkpoint(2)?;
            for _ in 0..100 {
                assert_eq!(execution.consult(), Ok(()));
                assert_eq!(work.cost(), (2, 2));
            }
            work.checkpoint(1)?;
            let expected = if refuse_charge {
                assert_eq!(work.checkpoint(1), Err(KernelStop::WorkCeiling));
                Err(WorkCeilingDenial::Stopped(KernelStop::WorkCeiling))
            } else {
                Ok(())
            };
            for _ in 0..100 {
                assert_eq!(execution.consult(), expected);
                assert_eq!(work.cost(), (3, 3));
            }
            Ok(())
        },
    );
    if refuse_charge {
        assert!(matches!(
            outcome.result,
            Err(crate::backend::ScopeStop::Failure(
                crate::backend::KernelFailure::Stop(KernelStop::WorkCeiling)
            ))
        ));
    } else {
        assert!(outcome.result.is_ok());
    }
    assert_eq!(outcome.report.charged_work(), 3);
}

#[test]
fn serial_work_spent_exactly_is_not_a_stop() {
    check_work_consultation(true, false);
}

#[test]
fn leased_work_spent_exactly_is_not_a_stop() {
    check_work_consultation(false, false);
}

#[test]
fn serial_refused_charge_is_a_stop_without_changing_charged_work() {
    check_work_consultation(true, true);
}

#[test]
fn leased_refused_charge_is_a_stop_without_changing_charged_work() {
    check_work_consultation(false, true);
}

#[test]
fn consultation_reads_inherited_cancellation_before_a_backing_deadline() {
    let _serial = TEST_LOCK.lock().unwrap();
    let source = CancellationSource::new();
    let parent = SerialRequest::from_policy(&request(1, 2_000, 3).policy, source.token(), None);
    let child = parent
        .clone()
        .with_cancellation(CancellationToken::new())
        .with_deadline(Some(Instant::now() - Duration::from_secs(1)));
    let outcome =
        crate::backend::run_scope_within::<(), (), _>(None, 0, 0, true, 3, Some(&parent), |work| {
            source.cancel();
            assert_eq!(
                ExecutionRequest::serial(&child).consult(),
                Err(WorkCeilingDenial::Stopped(KernelStop::Cancelled))
            );
            assert_eq!(work.cost(), (0, 0));
            Ok(())
        });
    assert_eq!(outcome.report.charged_work(), 0);
}

#[test]
fn consultation_preserves_a_nested_owners_latched_stop() {
    let _serial = TEST_LOCK.lock().unwrap();
    for (backend, workers) in [
        (BackendKind::Serial, 1),
        (BackendKind::Native, 1),
        (BackendKind::Native, 2),
        (BackendKind::Native, 4),
    ] {
        let cancellation = CancellationSource::new();
        let mut policy = request(workers, 2_000, 100);
        policy.cancellation = cancellation.token();
        let parent = authority().request_lease(policy).unwrap();
        let child = parent.child(request(1, 1_000, 100)).unwrap();
        let outer = run_checked_batch(Some(&parent), &batch(&[1], 0), backend, &|_, context| {
            context.checkpoint(1)?;
            let nested =
                run_checked_batch(Some(&child), &batch(&[1], 0), backend, &|_, context| {
                    context.checkpoint(2)?;
                    cancellation.cancel();
                    // The request becomes interrupted inside the nested owner.
                    // Consultation must lend that owner's real latch, not a fresh one.
                    for _ in 0..2 {
                        assert_eq!(
                            ExecutionRequest::leased(&child).consult(),
                            Err(WorkCeilingDenial::Stopped(KernelStop::Cancelled))
                        );
                    }
                    Ok::<_, KernelFailure<()>>(1_u64)
                });
            assert_eq!(
                nested.stop,
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Stop(KernelStop::NestedStopped),
                }),
                "the nested owner's swallowed interruption is reported once"
            );
            Ok::<_, KernelFailure<()>>(1_u64)
        });
        assert!(outer.values.is_empty());
        assert_eq!(
            outer.stop,
            Some(BatchStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: KernelFailure::Stop(KernelStop::NestedStopped),
            }),
            "consultation does not lose the invoking owner's latch"
        );
        assert_eq!(outer.report.charged_work(), 1 + 2);
        assert_eq!(outer.report.charged_span(), 1 + 2);
    }
}
