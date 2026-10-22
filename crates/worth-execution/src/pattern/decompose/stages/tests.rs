//! Refusals produced in reduction stages stay on the invoking request.
use super::*;
use crate::backend::{run_scope, KernelFailure, KernelStop, ScopeStop};
use crate::tests::{authority, request, TEST_LOCK};

fn assert_parent_stopped(result: Result<(), ScopeStop<()>>) {
    assert!(matches!(
        result,
        Err(ScopeStop::Failure(KernelFailure::Stop(
            KernelStop::NestedStopped
        )))
    ));
}

#[test]
fn frontier_admission_in_nested_decomposition_cannot_be_taken() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 1_200, 100)).unwrap();
    let outer = run_scope(None, 0, 0, |parent| {
        let inner = run_scope(Some(&lease), 0, 0, |context| {
            let decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
                vec![PartitionIdentity::new(1)],
                (),
                |_, _| (),
                0,
                0,
                100,
                0,
                0,
            )
            .unwrap();
            let map = ExecutionMap::<(), u64>::from_keyless_partitions(
                std::collections::BTreeMap::from([(
                    PartitionIdentity::new(1),
                    crate::KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )]),
            )
            .unwrap();
            let stage = Cell::new(DecomposeStage::Interior);
            let checked_changed = decomposition
                .check_changed::<_, _, ()>(
                    &map,
                    DecomposeKernelEditions {
                        interior: 1,
                        interface: 1,
                        back: 1,
                    },
                )
                .unwrap();
            let result = ExecutionDecompose::run_stages(
                checked_changed,
                Some(&lease),
                BackendKind::Serial,
                &|_, _| {
                    Ok::<_, MapKernelFailure<()>>(InteriorResult {
                        interior: (),
                        contribution: (),
                    })
                },
                &|_, _| {
                    Ok(InterfaceSolution {
                        solution: (),
                        slices: vec![()],
                    })
                },
                &|_, _| Ok(()),
                context,
                &stage,
                &Cell::new(None),
            );
            assert_eq!(stage.get(), DecomposeStage::Reduction);
            assert!(matches!(
                result,
                Err(DecomposeFailure::ScopeAdmission(
                    LeaseDenial::MemoryExhausted(_)
                ))
            ));
            result.map(|_| ()).map_err(|_| KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert_parent_stopped(outer.result);
}

#[test]
fn nested_decomposition_metric_overflow_cannot_be_taken() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outer = run_scope(None, 0, 0, |parent| {
        let inner = run_scope(None, 0, 0, |_| {
            let mut aggregate = ReductionMetrics::from_accounting(crate::ReductionAccounting {
                charged_span: u64::MAX,
                ..Default::default()
            });
            let edit = ReductionMetrics::from_accounting(crate::ReductionAccounting {
                charged_span: 1,
                ..Default::default()
            });
            let failure = add_metrics(&mut aggregate, edit).unwrap_err();
            assert_eq!(failure.reason, crate::ReductionRunStop::WorkCounterOverflow);
            Err::<(), _>(KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert_parent_stopped(outer.result);
}

#[test]
fn frontier_interruption_in_nested_decomposition_cannot_be_taken() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let source = crate::CancellationSource::new();
    let lease = authority()
        .request_lease(crate::LeaseRequest {
            cancellation: source.token(),
            ..request(1, 2_000, 100)
        })
        .unwrap();
    let outer = run_scope(None, 0, 0, |parent| {
        let inner = run_scope(None, 0, 0, |context| {
            let decomposition = ExecutionDecompose::<(), (), (), (), (), _>::try_new(
                vec![PartitionIdentity::new(1)],
                (),
                |_, _| {
                    source.cancel();
                },
                0,
                0,
                0,
                0,
                0,
            )
            .unwrap();
            let map = ExecutionMap::<(), u64>::from_keyless_partitions(
                std::collections::BTreeMap::from([(
                    PartitionIdentity::new(1),
                    crate::KeylessPartition {
                        value: (),
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    },
                )]),
            )
            .unwrap();
            let stage = Cell::new(DecomposeStage::Interior);
            let checked_changed = decomposition
                .check_changed::<_, _, ()>(
                    &map,
                    DecomposeKernelEditions {
                        interior: 1,
                        interface: 1,
                        back: 1,
                    },
                )
                .unwrap();
            let result = ExecutionDecompose::run_stages(
                checked_changed,
                Some(&lease),
                BackendKind::Serial,
                &|_, _| {
                    Ok::<_, MapKernelFailure<()>>(InteriorResult {
                        interior: (),
                        contribution: (),
                    })
                },
                &|_, _| {
                    Ok(InterfaceSolution {
                        solution: (),
                        slices: vec![()],
                    })
                },
                &|_, _| Ok(()),
                context,
                &stage,
                &Cell::new(None),
            );
            assert_eq!(stage.get(), DecomposeStage::Reduction);
            assert!(matches!(
                result,
                Err(DecomposeFailure::ReductionRun(ReductionRunFailure {
                    reason: crate::ReductionRunStop::Hook(KernelStop::Cancelled),
                    ..
                }))
            ));
            result.map(|_| ()).map_err(|_| KernelFailure::Domain(()))
        });
        assert!(inner.result.is_err());
        assert_eq!(parent.take_nested_computation_stop(), None);
        Ok::<_, KernelFailure<()>>(())
    });
    assert_parent_stopped(outer.result);
}
