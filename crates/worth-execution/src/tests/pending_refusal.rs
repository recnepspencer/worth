//! Refused questions become request stops only when their kernel fails.
use super::*;
use crate::ExecutionMap;

#[test]
fn a_real_nested_join_records_work_and_span_counter_overflow() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outer = crate::backend::run_scope(None, 0, 0, |parent| {
        let inner = crate::backend::run_scope(None, 0, 0, |child| {
            // The child inherited the remaining budget before a callback
            // accounts already-completed work on the invoking meter.
            child.checkpoint(u64::MAX)?;
            parent.account_completed_work(1)?;
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(inner.result.is_ok());
        assert_eq!(
            (inner.report.charged_work(), inner.report.charged_span()),
            (u64::MAX, u64::MAX)
        );
        assert_eq!(parent.take_nested_computation_stop(), None);
        assert_eq!(parent.cost(), (u64::MAX, u64::MAX));
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(matches!(
        outer.result,
        Err(crate::backend::ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (u64::MAX, u64::MAX)
    );
}

#[test]
fn declared_size_overflow_is_computation_owned_in_both_patterns() {
    use crate::{DecomposeKernelEditions, ExecutionDecompose, InterfaceSolution, InteriorResult};
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let input = ExecutionMap::<(), u64>::from_keyless_partitions(Default::default()).unwrap();
    for reduce in [true, false] {
        let outer = crate::backend::run_scope(None, 0, 0, |parent| {
            let inner = crate::backend::run_scope(None, 0, 0, |_| {
                if reduce {
                    let failure = input.certify_reduce(
                        &lease,
                        1,
                        |_, _| -> Result<(), KernelFailure<()>> {
                            panic!("declared size fails before kernels")
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
                } else {
                    let mut decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
                        vec![],
                        (),
                        |_, _| (),
                        0,
                        0,
                        0,
                        u64::MAX,
                        0,
                    )
                    .unwrap();
                    let failure = decomposition.run(
                        Some(&lease),
                        &input,
                        DecomposeKernelEditions {
                            interior: 1,
                            interface: 1,
                            back: 1,
                        },
                        |_, _| -> Result<InteriorResult<(), ()>, KernelFailure<()>> {
                            panic!("declared size fails before kernels")
                        },
                        |_, _| -> Result<InterfaceSolution<(), ()>, KernelFailure<()>> {
                            panic!("declared size fails before kernels")
                        },
                        |_, _| -> Result<(), KernelFailure<()>> {
                            panic!("declared size fails before kernels")
                        },
                    );
                    assert!(matches!(
                        failure,
                        Err(crate::DecomposeRunFailure {
                            cause: crate::DecomposeFailure::Input(
                                crate::DecomposeInputDenial::MemoryOverflow
                            ),
                            ..
                        })
                    ));
                }
                Err::<(), _>(KernelFailure::Domain(()))
            });
            assert!(inner.result.is_err());
            assert_eq!(
                parent.take_nested_computation_stop(),
                Some(crate::NestedComputationStop::DomainFailure)
            );
            Ok::<_, KernelFailure<()>>(())
        });
        assert!(outer.result.is_ok());
    }
}

#[test]
fn a_tolerated_memory_refusal_allows_the_child_and_parent_to_succeed() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let parent = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let child = parent.child(request(1, 1_000, 100)).unwrap();
    let outer = run_checked_batch(
        Some(&parent),
        &batch(&[1], 0),
        BackendKind::Serial,
        &|_, context| {
            let inner = run_checked_batch(
                Some(&child),
                &batch(&[1], 0),
                BackendKind::Serial,
                &|_, child_context| {
                    assert!(child.reserve_memory(u64::MAX).is_err());
                    assert_eq!(child_context.take_nested_computation_stop(), None);
                    child_context.checkpoint(1)?;
                    Ok::<_, KernelFailure<()>>(1_u64)
                },
            );
            assert_eq!(inner.stop, None);
            assert_eq!(inner.values, vec![1]);
            assert_eq!(context.take_nested_computation_stop(), None);
            context.checkpoint(1)?;
            Ok::<_, KernelFailure<()>>(1_u64)
        },
    );
    assert_eq!(outer.stop, None);
    assert_eq!(outer.values, vec![1]);
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (2, 2)
    );
}

#[test]
fn a_domain_failure_after_a_tolerated_refusal_is_request_owned() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let parent = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let child = parent.child(request(1, 1_000, 100)).unwrap();
    let outer = run_checked_batch(
        Some(&parent),
        &batch(&[1], 0),
        BackendKind::Serial,
        &|_, context| {
            let inner = run_checked_batch(
                Some(&child),
                &batch(&[1], 0),
                BackendKind::Serial,
                &|_, child_context| {
                    assert!(child.reserve_memory(u64::MAX).is_err());
                    child_context.checkpoint(1)?;
                    Err::<u64, _>(KernelFailure::Domain(7_u64))
                },
            );
            assert_eq!(
                inner.stop,
                Some(BatchStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: KernelFailure::Domain(7_u64)
                })
            );
            assert_eq!(context.take_nested_computation_stop(), None);
            Ok::<_, KernelFailure<()>>(1_u64)
        },
    );
    assert_eq!(
        outer.stop,
        Some(BatchStop::Failure {
            identity: PartitionIdentity::new(1),
            cause: KernelFailure::Stop(KernelStop::NestedStopped)
        })
    );
    assert_eq!(
        (outer.report.charged_work(), outer.report.charged_span()),
        (1, 1)
    );
}
