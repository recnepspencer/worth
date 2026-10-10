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

#[test]
fn repeated_consultation_reads_exhaustion_without_changing_the_meter() {
    let _serial = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(1, 2_000, 3)).unwrap();
    let serial = SerialRequest::from_policy(lease.policy(), CancellationToken::new(), None);
    for (execution, backing) in [
        (ExecutionRequest::serial(&serial), None),
        (ExecutionRequest::leased(&lease), Some(&lease)),
    ] {
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
                for _ in 0..100 {
                    assert_eq!(
                        execution.consult(),
                        Err(WorkCeilingDenial::Stopped(KernelStop::WorkCeiling))
                    );
                    assert_eq!(work.cost(), (3, 3));
                }
                Ok(())
            },
        );
        assert!(outcome.result.is_ok());
        assert_eq!(outcome.report.charged_work(), 3);
    }
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
