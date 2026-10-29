#[allow(unused_imports)] use crate::phase_two_neutral_certification::isolated_context::*;
use {std::num::NonZeroUsize, std::sync::Mutex, std::sync::OnceLock, std::time::Instant};

use {worth_execution::compare_canonical_values, worth_execution::BackInput, worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::CanonicalBits, worth_execution::ChargedBytes, worth_execution::DecomposeFailure, worth_execution::DecomposeKernelEditions, worth_execution::EquivalencePredicate, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionDecompose, worth_execution::ExecutionForkJoin, worth_execution::ExecutionMap, worth_execution::ExecutionRounds, worth_execution::ExecutionScan, worth_execution::ForkChild, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop, worth_execution::RoundsOutcome, worth_execution::ScanOutcome};
use {worth_foundational::DeterminismContract, worth_foundational::EquivalenceContractId, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

#[test]
fn map_and_fork_match_independent_values_under_widths_and_perturbed_schedules() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let map = integer_map(31);
    let expected: Vec<_> = (1..=31_u64).map(|n| n * n + 7).collect();
    let serial = map.run(None, |n, context| {
        context.checkpoint(*n % 5 + 1)?;
        Ok::<_, MapKernelFailure<()>>(n * n + 7)
    });
    assert!(matches!(&serial, worth_execution::MapOutcome::Complete { values, .. } if values == &expected));
    let serial_work = serial.report().charged_work();
    let serial_span = serial.report().charged_span();
    assert_eq!(serial_work, (1..=31_u64).map(|n| n % 5 + 1).sum::<u64>());
    assert_eq!(serial_span, 5);

    let ids = [3_u64, 5, 8].map(worth_foundational::PartitionIdentity::new);
    let fork = worth_execution::ExecutionForkJoin::try_from_children(
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
                worth_foundational::ExecutionPosture::Automatic,
                width,
                200,
                worth_execution::CancellationToken::new(),
                None,
                worth_foundational::DeterminismContract::CanonicalBitwise,
            );
            let certified = map
                .certify(&lease, seed, |n, context| {
                    context.checkpoint(*n % 5 + 1)?;
                    Ok::<_, MapKernelFailure<()>>(n * n + 7)
                })
                .unwrap();
            assert!(
                matches!(&certified, worth_execution::MapOutcome::Complete { values, .. } if values == &expected)
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
            assert!(matches!(floating, worth_execution::MapOutcome::Complete { values, .. }
                if values.iter().map(|value| value.to_bits()).collect::<Vec<_>>() == adversarial_bits));

            let joined = fork
                .certify(&lease, seed, |n, context| {
                    context.checkpoint(2)?;
                    Ok::<_, MapKernelFailure<()>>(n * n)
                })
                .unwrap();
            assert!(
                matches!(&joined, worth_execution::MapOutcome::Complete { values, .. } if values == &vec![9, 25, 64])
            );
            assert_eq!(joined.report().charged_work(), 6);
            assert_eq!(joined.report().charged_span(), 2);
        }
    }
}


#[test]
fn ordered_scan_and_rounds_stop_at_their_canonical_barriers() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let ids: Vec<_> = (1..=4).map(worth_foundational::PartitionIdentity::new).collect();
    let scan = worth_execution::ExecutionScan::try_from_ordered(
        ids.clone(),
        ids.into_iter().map(|id| (id, id.value())).collect(),
    )
    .unwrap();
    let limited = lease(
        worth_foundational::ExecutionPosture::Automatic,
        4,
        2,
        worth_execution::CancellationToken::new(),
        None,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    let stopped = scan.run(Some(&limited), 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        let next = carry + item;
        Ok::<_, MapKernelFailure<()>>((next, next))
    });
    assert!(matches!(stopped, worth_execution::ScanOutcome::Stopped {
        completed_state: 3, completed_prefix, boundary: Some(id),
        reason: worth_execution::MapStop::WorkExhausted { .. }, ..
    } if completed_prefix == vec![1, 3] && id == worth_foundational::PartitionIdentity::new(3)));

    let cancellation = worth_execution::CancellationSource::new();
    let round_lease = lease(
        worth_foundational::ExecutionPosture::Automatic,
        4,
        10,
        cancellation.token(),
        None,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    let rounds = worth_execution::ExecutionRounds::try_new(std::num::NonZeroUsize::new(5).unwrap()).unwrap();
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
    assert!(matches!(stopped, worth_execution::RoundsOutcome::Stopped {
        completed_state: 1, completed_rounds: 1, boundary: Some(id),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled), .. }, ..
    } if id == worth_foundational::PartitionIdentity::new(2)));
}

#[path = "run_only_patterns/mod.rs"]
pub(crate) mod run_only_patterns;
#[path = "sparse_and_stops/mod.rs"]
pub(crate) mod sparse_and_stops;
