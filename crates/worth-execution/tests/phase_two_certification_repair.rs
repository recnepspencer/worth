use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

use worth_execution::{
    CancellationSource, CancellationToken, CanonicalBits, ChargedBytes,
    DecomposeCertificationFailure, DecomposeKernelEditions, EquivalencePredicate,
    ExecutionAuthority, ExecutionAuthorityConfig, ExecutionDecompose, ExecutionMap,
    InterfaceSolution, InteriorResult, LeaseRequest, MapKernelFailure, MapOutcome, MapPartition,
    OracleMismatch, ReduceCertificationFailure,
};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy, PartitionIdentity,
};

const CONTRACT: EquivalenceContractId = EquivalenceContractId::new(41);
static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[path = "phase_two_certification_repair/clone_panic.rs"]
mod clone_panic;
#[path = "phase_two_certification_repair/retained_captures.rs"]
mod retained_captures;

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct_with_equivalences(
            ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(1 << 20),
            },
            [EquivalencePredicate::new(
                CONTRACT,
                [41; 32],
                |left, right| {
                    let (Ok(left), Ok(right)): (Result<[u8; 8], _>, Result<[u8; 8], _>) =
                        (left.try_into(), right.try_into())
                    else {
                        return false;
                    };
                    (f64::from_le_bytes(left) - f64::from_le_bytes(right)).abs() < 0.01
                },
            )],
        )
        .unwrap()
    })
}

fn lease(
    work: u64,
    memory: u64,
    determinism: DeterminismContract,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                determinism,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, work),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

fn map(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(PartitionIdentity::new).collect();
    let parts = ids
        .iter()
        .copied()
        .map(|identity| MapPartition {
            identity,
            value: identity.value(),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 8,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(ids, parts).unwrap()
}

#[test]
fn nested_reduce_certification_charges_only_selected_work() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let inner = map(3);
    let outer = map(1);
    let run = |work| {
        let lease = lease(work, 1 << 20, DeterminismContract::CanonicalBitwise);
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
        MapOutcome::Complete { values, report } => {
            assert_eq!(report.charged_work(), values[0]);
            values[0]
        }
        _ => panic!("unexpected nested outcome"),
    };
    let tight = run(selected);
    assert!(
        matches!(tight, MapOutcome::Complete { report, .. } if report.charged_work() == selected)
    );
}

#[test]
fn reduce_uses_installed_equivalence_and_bitwise_policy() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let run = |determinism| {
        let lease = lease(100, 1 << 20, determinism);
        let calls = AtomicUsize::new(0);
        map(1).certify_reduce(
            &lease,
            9,
            |_, context| {
                context.checkpoint(1)?;
                let value = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
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
    assert!(run(DeterminismContract::ContractEquivalent(CONTRACT)).is_ok());
    assert!(matches!(
        run(DeterminismContract::CanonicalBitwise),
        Err(ReduceCertificationFailure::Mismatch(OracleMismatch::Values))
    ));
}

#[test]
fn decompose_uses_installed_equivalence_and_bitwise_policy() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let run = |determinism| {
        let lease = lease(100, 1 << 20, determinism);
        let calls = AtomicUsize::new(0);
        let mut decomposition = ExecutionDecompose::try_new(
            vec![PartitionIdentity::new(1)],
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
                let value = if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                    1.0
                } else {
                    1.001
                };
                Ok::<_, MapKernelFailure<()>>(value)
            },
        )
    };
    let equivalent = run(DeterminismContract::ContractEquivalent(CONTRACT));
    assert!(equivalent.is_ok(), "{:?}", equivalent.err());
    assert!(matches!(
        run(DeterminismContract::CanonicalBitwise),
        Err(DecomposeCertificationFailure::Mismatch(
            OracleMismatch::Values
        ))
    ));
}

#[test]
fn nested_decompose_certification_charges_only_selected_work() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let outer = map(1);
    let changed = map(1);
    let run = |work| {
        let lease = lease(work, 1 << 20, DeterminismContract::CanonicalBitwise);
        outer.run(Some(&lease), |_, context| {
            let mut decomposition = ExecutionDecompose::try_new(
                vec![PartitionIdentity::new(1)],
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
        MapOutcome::Complete { values, report } => {
            assert_eq!(report.charged_work(), values[0]);
            values[0]
        }
        _ => panic!("nested decomposition stopped"),
    };
    assert!(matches!(run(selected), MapOutcome::Complete { report, .. }
        if report.charged_work() == selected));
}

#[test]
fn certification_memory_peak_includes_retained_results() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let reduce = |memory| {
        let lease = lease(100, memory, DeterminismContract::CanonicalBitwise);
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
        let lease = lease(100, memory, DeterminismContract::CanonicalBitwise);
        let mut decomposition = ExecutionDecompose::try_new(
            vec![PartitionIdentity::new(1)],
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
