use std::{
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
    time::Instant,
};

use worth_execution::{
    compare_canonical_values, BackInput, CancellationSource, CancellationToken, CanonicalBits,
    ChargedBytes, DecomposeFailure, DecomposeKernelEditions, EquivalencePredicate,
    ExecutionAuthority, ExecutionAuthorityConfig, ExecutionDecompose, ExecutionForkJoin,
    ExecutionMap, ExecutionRounds, ExecutionScan, ForkChild, InterfaceSolution, InteriorResult,
    LeaseRequest, MapKernelFailure, MapKernelStop, MapOutcome, MapPartition, MapStop,
    RoundsOutcome, ScanOutcome,
};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy, PartitionIdentity,
};

const MEMORY: u64 = 1 << 20;
const EQUIVALENCE_ID: EquivalenceContractId = EquivalenceContractId::new(91);
const EDITIONS: DecomposeKernelEditions = DecomposeKernelEditions {
    interior: 1,
    interface: 1,
    back: 1,
};
static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct_with_equivalences(
            ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(machine_width().saturating_add(1).max(4)).unwrap(),
                charged_memory_bytes: MEMORY,
            },
            [EquivalencePredicate::new(
                EQUIVALENCE_ID,
                [91; 32],
                |left, right| {
                    let (Ok(left), Ok(right)): (Result<[u8; 8], _>, Result<[u8; 8], _>) =
                        (left.try_into(), right.try_into())
                    else {
                        return false;
                    };
                    let left = f64::from_le_bytes(left);
                    let right = f64::from_le_bytes(right);
                    left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-12
                },
            )],
        )
        .unwrap()
    })
}

fn machine_width() -> usize {
    std::thread::available_parallelism().map_or(1, NonZeroUsize::get)
}

fn widths() -> Vec<usize> {
    let machine = machine_width();
    let mut widths = vec![1, 2, machine, machine.saturating_add(1)];
    widths.sort_unstable();
    widths.dedup();
    widths
}

fn execution_matrix() -> Vec<(ExecutionPosture, usize)> {
    let mut matrix = vec![(ExecutionPosture::Serial, 1)];
    matrix.extend(
        widths()
            .into_iter()
            .map(|width| (ExecutionPosture::Automatic, width)),
    );
    matrix
}

fn lease(
    posture: ExecutionPosture,
    width: usize,
    work: u64,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
    determinism: DeterminismContract,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                determinism,
                ExecutionBudget::new(NonZeroUsize::new(width).unwrap(), MEMORY, work),
            ),
            deadline,
            cancellation,
        })
        .unwrap()
}

fn integer_map(count: u64) -> ExecutionMap<u64, u64> {
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
fn map_and_fork_match_independent_values_under_widths_and_perturbed_schedules() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let map = integer_map(31);
    let expected: Vec<_> = (1..=31_u64).map(|n| n * n + 7).collect();
    let serial = map.run(None, |n, context| {
        context.checkpoint(*n % 5 + 1)?;
        Ok::<_, MapKernelFailure<()>>(n * n + 7)
    });
    assert!(matches!(&serial, MapOutcome::Complete { values, .. } if values == &expected));
    let serial_work = serial.report().charged_work();
    let serial_span = serial.report().charged_span();
    assert_eq!(serial_work, (1..=31_u64).map(|n| n % 5 + 1).sum::<u64>());
    assert_eq!(serial_span, 5);

    let ids = [3_u64, 5, 8].map(PartitionIdentity::new);
    let fork = ExecutionForkJoin::try_from_children(
        ids.to_vec(),
        ids.into_iter()
            .map(|identity| ForkChild {
                identity,
                input: identity.value(),
                read_keys: Vec::<u64>::new(),
                write_keys: vec![identity.value()],
                kernel_scratch_bytes: 0,
                max_result_bytes: 8,
            })
            .collect(),
    )
    .unwrap();
    let adversarial_bits = [
        (-0.0_f64).to_bits(),
        0.0_f64.to_bits(),
        0x7ff8_0000_0000_0042,
        0x7ff8_0000_0000_0043,
    ];
    for width in widths() {
        for seed in [1, 0x9e37_79b9, 0xdead_beef] {
            let lease = lease(
                ExecutionPosture::Automatic,
                width,
                200,
                CancellationToken::new(),
                None,
                DeterminismContract::CanonicalBitwise,
            );
            let certified = map
                .certify(&lease, seed, |n, context| {
                    context.checkpoint(*n % 5 + 1)?;
                    Ok::<_, MapKernelFailure<()>>(n * n + 7)
                })
                .unwrap();
            assert!(
                matches!(&certified, MapOutcome::Complete { values, .. } if values == &expected)
            );
            assert_eq!(certified.report().charged_work(), serial_work);
            assert_eq!(certified.report().charged_span(), serial_span);

            let floating = integer_map(4)
                .certify(&lease, seed, |n, context| {
                    context.checkpoint(1)?;
                    Ok::<_, MapKernelFailure<()>>(f64::from_bits(
                        adversarial_bits[(*n - 1) as usize],
                    ))
                })
                .unwrap();
            assert!(matches!(floating, MapOutcome::Complete { values, .. }
                if values.iter().map(|value| value.to_bits()).collect::<Vec<_>>() == adversarial_bits));

            let joined = fork
                .certify(&lease, seed, |n, context| {
                    context.checkpoint(2)?;
                    Ok::<_, MapKernelFailure<()>>(n * n)
                })
                .unwrap();
            assert!(
                matches!(&joined, MapOutcome::Complete { values, .. } if values == &vec![9, 25, 64])
            );
            assert_eq!(joined.report().charged_work(), 6);
            assert_eq!(joined.report().charged_span(), 2);
        }
    }
}

#[path = "phase_two_neutral_certification/run_only_patterns.rs"]
mod run_only_patterns;

#[test]
fn ordered_scan_and_rounds_stop_at_their_canonical_barriers() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let ids: Vec<_> = (1..=4).map(PartitionIdentity::new).collect();
    let scan = ExecutionScan::try_from_ordered(
        ids.clone(),
        ids.into_iter().map(|id| (id, id.value())).collect(),
    )
    .unwrap();
    let limited = lease(
        ExecutionPosture::Automatic,
        4,
        2,
        CancellationToken::new(),
        None,
        DeterminismContract::CanonicalBitwise,
    );
    let stopped = scan.run(Some(&limited), 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        let next = carry + item;
        Ok::<_, MapKernelFailure<()>>((next, next))
    });
    assert!(matches!(stopped, ScanOutcome::Stopped {
        completed_state: 3, completed_prefix, boundary: Some(id),
        reason: MapStop::WorkExhausted { .. }, ..
    } if completed_prefix == vec![1, 3] && id == PartitionIdentity::new(3)));

    let cancellation = CancellationSource::new();
    let round_lease = lease(
        ExecutionPosture::Automatic,
        4,
        10,
        cancellation.token(),
        None,
        DeterminismContract::CanonicalBitwise,
    );
    let rounds = ExecutionRounds::try_new(NonZeroUsize::new(5).unwrap()).unwrap();
    let stopped = rounds.run(
        Some(&round_lease),
        0_u64,
        0,
        0,
        0,
        |prior, round, context| {
            context.checkpoint(1)?;
            if round == 1 {
                cancellation.cancel();
            }
            Ok::<_, MapKernelFailure<()>>(prior + 1)
        },
        |_, _| false,
    );
    assert!(matches!(stopped, RoundsOutcome::Stopped {
        completed_state: 1, completed_rounds: 1, boundary: Some(id),
        reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. }, ..
    } if id == PartitionIdentity::new(2)));
}

#[path = "phase_two_neutral_certification/sparse_and_stops.rs"]
mod sparse_and_stops;
