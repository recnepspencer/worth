#[allow(unused_imports)]
use crate::phase_two_certification_repair::isolated_context::*;
use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::Mutex, std::sync::OnceLock,
};

use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::CanonicalBits, worth_execution::ChargedBytes,
    worth_execution::DecomposeCertificationFailure, worth_execution::DecomposeKernelEditions,
    worth_execution::EquivalencePredicate, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionDecompose,
    worth_execution::ExecutionMap, worth_execution::InterfaceSolution,
    worth_execution::InteriorResult, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition,
    worth_execution::OracleMismatch, worth_execution::ReduceCertificationFailure,
};
use {
    worth_foundational::DeterminismContract, worth_foundational::EquivalenceContractId,
    worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture,
    worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity,
};

#[test]
fn nested_reduce_certification_charges_only_selected_work() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let inner = map(3);
    let outer = map(1);
    let run = |work| {
        let lease = lease(
            work,
            1 << 20,
            worth_foundational::DeterminismContract::CanonicalBitwise,
        );
        outer.run(Some(&lease), |_, context| {
            let (_, report, _) = inner
                .certify_reduce(
                    &lease,
                    7,
                    |value, child| {
                        child.checkpoint(1)?;
                        Ok::<_, MapKernelFailure<()>>(*value)
                    },
                    0_u64,
                    |left, right| left + right,
                    8,
                    0,
                )
                .expect("nested reduction must complete");
            context.checkpoint(0)?;
            Ok::<_, MapKernelFailure<()>>(report.charged_work())
        })
    };
    let baseline = run(100);
    let selected = match &baseline {
        worth_execution::MapOutcome::Complete { values, report } => {
            assert_eq!(report.charged_work(), values[0]);
            values[0]
        }
        _ => panic!("unexpected nested outcome"),
    };
    let tight = run(selected);
    assert!(
        matches!(tight, worth_execution::MapOutcome::Complete { report, .. } if report.charged_work() == selected)
    );
}

#[test]
fn reduce_uses_installed_equivalence_and_bitwise_policy() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let run = |determinism| {
        let lease = lease(100, 1 << 20, determinism);
        let calls = std::sync::atomic::AtomicUsize::new(0);
        map(1).certify_reduce(
            &lease,
            9,
            |_, context| {
                context.checkpoint(1)?;
                let value = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                    1.0
                } else {
                    1.001
                };
                Ok::<_, MapKernelFailure<()>>(value)
            },
            0.0_f64,
            |left, right| left + right,
            8,
            0,
        )
    };
    assert!(
        run(worth_foundational::DeterminismContract::ContractEquivalent(
            CONTRACT
        ))
        .is_ok()
    );
    assert!(matches!(
        run(worth_foundational::DeterminismContract::CanonicalBitwise),
        Err(worth_execution::ReduceCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Values
        ))
    ));
}

#[test]
fn decompose_uses_installed_equivalence_and_bitwise_policy() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let run = |determinism| {
        let lease = lease(100, 1 << 20, determinism);
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let mut decomposition = worth_execution::ExecutionDecompose::try_new(
            vec![worth_foundational::PartitionIdentity::new(1)],
            0.0_f64,
            |left: &f64, right: &f64| left + right,
            8,
            8,
            8,
            256,
            0,
        )
        .unwrap();
        decomposition.certify(
            &lease,
            11,
            &map(1),
            DecomposeKernelEditions {
                interior: 1,
                interface: 1,
                back: 1,
            },
            |value, context| {
                context.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(InteriorResult {
                    interior: *value,
                    contribution: *value as f64,
                })
            },
            |_, context| {
                context.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(InterfaceSolution {
                    solution: 0.0_f64,
                    slices: vec![0.0_f64],
                })
            },
            |_, context| {
                context.checkpoint(1)?;
                let value = if calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                    1.0
                } else {
                    1.001
                };
                Ok::<_, MapKernelFailure<()>>(value)
            },
        )
    };
    let equivalent = run(worth_foundational::DeterminismContract::ContractEquivalent(
        CONTRACT,
    ));
    assert!(equivalent.is_ok(), "{:?}", equivalent.err());
    assert!(matches!(
        run(worth_foundational::DeterminismContract::CanonicalBitwise),
        Err(worth_execution::DecomposeCertificationFailure::Mismatch(
            worth_execution::OracleMismatch::Values
        ))
    ));
}

#[test]
fn nested_decompose_certification_charges_only_selected_work() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outer = map(1);
    let changed = map(1);
    let run = |work| {
        let lease = lease(
            work,
            1 << 20,
            worth_foundational::DeterminismContract::CanonicalBitwise,
        );
        outer.run(Some(&lease), |_, context| {
            let mut decomposition = worth_execution::ExecutionDecompose::try_new(
                vec![worth_foundational::PartitionIdentity::new(1)],
                0_u64,
                |left: &u64, right: &u64| left + right,
                8,
                8,
                8,
                256,
                0,
            )
            .unwrap();
            let result = decomposition
                .certify(
                    &lease,
                    1,
                    &changed,
                    DecomposeKernelEditions {
                        interior: 1,
                        interface: 1,
                        back: 1,
                    },
                    |value, child| {
                        child.checkpoint(1)?;
                        Ok::<_, MapKernelFailure<()>>(InteriorResult {
                            interior: *value,
                            contribution: *value,
                        })
                    },
                    |value, child| {
                        child.checkpoint(1)?;
                        Ok::<_, MapKernelFailure<()>>(InterfaceSolution {
                            solution: *value,
                            slices: vec![*value],
                        })
                    },
                    |input, child| {
                        child.checkpoint(1)?;
                        Ok::<_, MapKernelFailure<()>>(input.interior + input.interface_slice)
                    },
                )
                .expect("nested decomposition must complete");
            context.checkpoint(0)?;
            Ok::<_, MapKernelFailure<()>>(result.total_report.charged_work())
        })
    };
    let selected = match run(100) {
        worth_execution::MapOutcome::Complete { values, report } => {
            assert_eq!(report.charged_work(), values[0]);
            values[0]
        }
        _ => panic!("nested decomposition stopped"),
    };
    assert!(
        matches!(run(selected), worth_execution::MapOutcome::Complete { report, .. }
        if report.charged_work() == selected)
    );
}

#[test]
fn certification_memory_peak_includes_retained_results() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let reduce = |memory| {
        let lease = lease(
            100,
            memory,
            worth_foundational::DeterminismContract::CanonicalBitwise,
        );
        map(3).certify_reduce(
            &lease,
            1,
            |value, child| {
                child.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(*value)
            },
            0_u64,
            |left, right| left + right,
            8,
            0,
        )
    };
    let (_, report, _) = reduce(1 << 20).unwrap();
    let peak = report.physical().peak_charged_memory_bytes();
    assert!(peak > 0);
    assert!(reduce(peak - 1).is_err());

    let decompose = |memory| {
        let lease = lease(
            100,
            memory,
            worth_foundational::DeterminismContract::CanonicalBitwise,
        );
        let mut decomposition = worth_execution::ExecutionDecompose::try_new(
            vec![worth_foundational::PartitionIdentity::new(1)],
            0_u64,
            |left: &u64, right: &u64| left + right,
            8,
            8,
            8,
            256,
            0,
        )
        .unwrap();
        decomposition.certify(
            &lease,
            1,
            &map(1),
            DecomposeKernelEditions {
                interior: 1,
                interface: 1,
                back: 1,
            },
            |value, child| {
                child.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(InteriorResult {
                    interior: *value,
                    contribution: *value,
                })
            },
            |value, child| {
                child.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(InterfaceSolution {
                    solution: *value,
                    slices: vec![*value],
                })
            },
            |input, child| {
                child.checkpoint(1)?;
                Ok::<_, MapKernelFailure<()>>(input.interior + input.interface_slice)
            },
        )
    };
    let result = decompose(1 << 20).unwrap();
    let peak = result.total_report.physical().peak_charged_memory_bytes();
    assert!(peak > 0);
    assert!(decompose(peak - 1).is_err());
}
#[path = "clone_panic/mod.rs"]
pub(crate) mod clone_panic;
#[path = "retained_captures/mod.rs"]
pub(crate) mod retained_captures;
