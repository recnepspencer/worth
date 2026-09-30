use super::*;

#[test]
fn reduction_scan_and_rounds_preserve_canonical_values_and_cost() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let map = integer_map(7);
    let floating = [1e16, 1.0, -1e16, -0.0, 3.0, 1e-16, -3.0];
    let reduce = |lease: Option<&worth_execution::ExecutionResourceLease<'_>>| {
        let (tree, report, metrics) = map
            .run_reduce(
                lease,
                |n, context| {
                    context.checkpoint(1)?;
                    for _ in 0..(*n % 4) {
                        std::thread::yield_now();
                    }
                    Ok::<_, MapKernelFailure<()>>(floating[(*n - 1) as usize])
                },
                0.0_f64,
                |left, right| {
                    for _ in 0..(left.to_bits() % 4) {
                        std::thread::yield_now();
                    }
                    left + right
                },
                8,
                0,
            )
            .unwrap();
        (
            tree.result().to_bits(),
            report.charged_work(),
            report.charged_span(),
            metrics,
        )
    };
    let serial_reduced = reduce(None);
    assert_eq!(serial_reduced.1, 7 + serial_reduced.3.charged_work);
    assert!(serial_reduced.2 < serial_reduced.1);

    let expected_prefix: Vec<_> = (1..=17_u64).map(|n| n * (n + 1) / 2).collect();
    let rounds = ExecutionRounds::try_new(NonZeroUsize::new(19).unwrap()).unwrap();
    let mut serial_scan = None;
    let mut serial_rounds = None;
    for (posture, width) in execution_matrix() {
        let lease = lease(
            posture,
            width,
            100,
            CancellationToken::new(),
            None,
            DeterminismContract::CanonicalBitwise,
        );
        let reduced = reduce(Some(&lease));
        assert_eq!(reduced.0, serial_reduced.0);
        assert_eq!(reduced.1, serial_reduced.1);
        assert_eq!(reduced.2, serial_reduced.2);
        let (certified, certified_report, _) = map
            .certify_reduce(
                &lease,
                0x9e37_79b9 ^ width as u64,
                |n, context| {
                    context.checkpoint(1)?;
                    Ok::<_, MapKernelFailure<()>>(floating[(*n - 1) as usize])
                },
                0.0_f64,
                |left, right| left + right,
                8,
                0,
            )
            .unwrap();
        assert_eq!(certified.result().to_bits(), serial_reduced.0);
        assert_eq!(certified_report.charged_work(), serial_reduced.1);
        assert_eq!(certified_report.charged_span(), serial_reduced.2);
        let ids: Vec<_> = (1..=17).map(PartitionIdentity::new).collect();
        let scan = ExecutionScan::try_from_ordered(
            ids.clone(),
            ids.into_iter().map(|id| (id, id.value())).collect(),
        )
        .unwrap();
        let scanned = scan.run(Some(&lease), 0_u64, 0, 0, 0, 0, |carry, item, context| {
            context.checkpoint(1)?;
            for _ in 0..(item % 4) {
                std::thread::yield_now();
            }
            let next = carry + item;
            Ok::<_, MapKernelFailure<()>>((next, next))
        });
        assert!(
            matches!(&scanned, ScanOutcome::Complete { state: 153, prefixes, .. } if prefixes == &expected_prefix)
        );
        assert_eq!(scanned.report().charged_work(), 17);
        assert_eq!(scanned.report().charged_span(), 17);
        let scan_signature = match &scanned {
            ScanOutcome::Complete {
                state, prefixes, ..
            } => (
                *state,
                prefixes.clone(),
                scanned.report().charged_work(),
                scanned.report().charged_span(),
            ),
            ScanOutcome::Stopped { .. } => panic!("scan unexpectedly stopped"),
        };
        if let Some(serial) = &serial_scan {
            assert_eq!(&scan_signature, serial);
        } else {
            serial_scan = Some(scan_signature);
        }
        let converged = rounds.run(
            Some(&lease),
            0_u64,
            0,
            0,
            0,
            |prior, _, context| {
                context.checkpoint(1)?;
                for _ in 0..(prior % 4) {
                    std::thread::yield_now();
                }
                Ok::<_, MapKernelFailure<()>>(prior + 1)
            },
            |_, next| *next == 13,
        );
        assert!(matches!(
            converged,
            RoundsOutcome::Converged {
                state: 13,
                rounds: 13,
                ..
            }
        ));
        assert_eq!(converged.report().charged_work(), 13);
        let rounds_signature = match &converged {
            RoundsOutcome::Converged { state, rounds, .. } => (
                *state,
                *rounds,
                converged.report().charged_work(),
                converged.report().charged_span(),
            ),
            _ => panic!("rounds did not converge"),
        };
        if let Some(serial) = serial_rounds {
            assert_eq!(rounds_signature, serial);
        } else {
            serial_rounds = Some(rounds_signature);
        }
    }
}

#[test]
fn ordered_patterns_preserve_cost_with_seeded_nested_fork_schedules() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let identities: Vec<_> = (1..=3).map(PartitionIdentity::new).collect();
    let fork = ExecutionForkJoin::try_from_children(
        identities.clone(),
        identities
            .into_iter()
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
    let mut serial_scan = None;
    let mut serial_rounds = None;
    for (posture, width) in execution_matrix() {
        let lease = lease(
            posture,
            width,
            100,
            CancellationToken::new(),
            None,
            DeterminismContract::CanonicalBitwise,
        );
        let ids: Vec<_> = (1..=7).map(PartitionIdentity::new).collect();
        let scan = ExecutionScan::try_from_ordered(
            ids.clone(),
            ids.into_iter().map(|id| (id, id.value())).collect(),
        )
        .unwrap();
        let scan_outcome = scan.run(Some(&lease), 0_u64, 0, 0, 0, 0, |prior, item, context| {
            context.checkpoint(1)?;
            let nested = fork
                .certify(&lease, 0x1000 ^ *item, |n, child| {
                    child.checkpoint(1)?;
                    Ok::<_, MapKernelFailure<()>>(n * item)
                })
                .expect("nested fork oracle mismatch");
            let MapOutcome::Complete { values, .. } = nested else {
                panic!("nested fork stopped");
            };
            let next = prior + values.into_iter().sum::<u64>();
            Ok::<_, MapKernelFailure<()>>((next, next))
        });
        let scan_signature = match scan_outcome {
            ScanOutcome::Complete {
                state,
                prefixes,
                report,
            } => {
                assert_eq!(state, 168);
                assert_eq!(
                    prefixes,
                    (1..=7_u64).map(|n| 3 * n * (n + 1)).collect::<Vec<_>>()
                );
                (
                    state,
                    prefixes,
                    report.charged_work(),
                    report.charged_span(),
                )
            }
            ScanOutcome::Stopped { .. } => panic!("scan stopped"),
        };
        if let Some(serial) = &serial_scan {
            assert_eq!(&scan_signature, serial);
        } else {
            serial_scan = Some(scan_signature);
        }

        let rounds = ExecutionRounds::try_new(NonZeroUsize::new(5).unwrap()).unwrap();
        let rounds_outcome = rounds.run(
            Some(&lease),
            0_u64,
            0,
            0,
            0,
            |prior, round, context| {
                context.checkpoint(1)?;
                let nested = fork
                    .certify(&lease, 0x2000 ^ round as u64, |n, child| {
                        child.checkpoint(1)?;
                        Ok::<_, MapKernelFailure<()>>(*n)
                    })
                    .expect("nested fork oracle mismatch");
                let MapOutcome::Complete { values, .. } = nested else {
                    panic!("nested fork stopped");
                };
                Ok::<_, MapKernelFailure<()>>(prior + values.into_iter().sum::<u64>())
            },
            |_, next| *next == 30,
        );
        let rounds_signature = match rounds_outcome {
            RoundsOutcome::Converged {
                state,
                rounds,
                report,
            } => {
                assert_eq!((state, rounds), (30, 5));
                (state, rounds, report.charged_work(), report.charged_span())
            }
            _ => panic!("rounds did not converge"),
        };
        if let Some(serial) = serial_rounds {
            assert_eq!(rounds_signature, serial);
        } else {
            serial_rounds = Some(rounds_signature);
        }
    }
}
