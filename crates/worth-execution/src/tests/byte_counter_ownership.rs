//! Byte-counter facts remain latched even when a kernel swallows their errors.
use super::*;
use crate::backend::{run_scope, KernelContext, ScopeStop};

fn swallowed_overflow(produce: impl FnOnce()) -> (KernelStop, (u64, u64)) {
    let outer = run_scope(None, 0, 0, |grandparent| {
        let inner = run_scope(None, 0, 0, |kernel| {
            produce();
            assert_eq!(kernel.take_nested_computation_stop(), None);
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(matches!(
            inner.result,
            Err(ScopeStop::Failure(KernelFailure::Stop(
                KernelStop::NestedStopped
            )))
        ));
        assert_eq!(grandparent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    let Err(ScopeStop::Failure(KernelFailure::Stop(stop))) = outer.result else {
        panic!("a swallowed byte overflow must stop the request");
    };
    assert_eq!(stop, KernelStop::NestedStopped);
    (
        stop,
        (outer.report.charged_work(), outer.report.charged_span()),
    )
}

#[test]
fn the_same_byte_overflow_is_a_fact_through_batch_and_oracle_doors() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 1_000, 100)).unwrap();
    let declared =
        AdmittedBatch::try_admit(vec![(PartitionIdentity::new(1), 1_u64, 0, u64::MAX)], 0).unwrap();
    let kernel = |_: &u64, _: &mut KernelContext<'_, '_>| -> Result<u64, KernelFailure<()>> {
        panic!("byte arithmetic fails before either kernel")
    };
    let batch_outcome = swallowed_overflow(|| {
        let outcome = run_checked_batch(Some(&lease), &declared, BackendKind::Serial, &kernel);
        assert_eq!(
            outcome.stop,
            Some(BatchStop::Admission(LeaseDenial::ChargedBytesOverflow))
        );
    });
    let oracle_outcome = swallowed_overflow(|| {
        let outcome = crate::oracle::certify(&lease, &declared, BackendKind::Serial, &kernel);
        assert!(matches!(outcome, Err(crate::oracle::OracleMismatch::Stop)));
    });
    assert_eq!(oracle_outcome, batch_outcome);
    assert_eq!(batch_outcome, (KernelStop::NestedStopped, (0, 0)));
}

#[test]
fn a_swallowed_reduce_certification_byte_overflow_latches_the_request() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 1_000, 100)).unwrap();
    let input =
        crate::ExecutionMap::<(), u64>::from_keyless_partitions(Default::default()).unwrap();
    let outcome = swallowed_overflow(|| {
        let failure = input.certify_reduce(
            &lease,
            1,
            |_, _| -> Result<(), KernelFailure<()>> {
                panic!("byte arithmetic fails before either kernel")
            },
            (),
            |_, _| (),
            u64::MAX,
            1,
        );
        assert!(matches!(
            failure,
            Err(crate::ReduceCertificationFailure::Mismatch(
                crate::oracle::OracleMismatch::Stop
            ))
        ));
    });
    assert_eq!(outcome, (KernelStop::NestedStopped, (0, 0)));
}
