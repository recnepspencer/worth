use super::*;

#[test]
fn stage_stops_keep_canonical_prefix_and_do_not_publish_partial_decomposition() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let map = integer_map(4);
    let cancelled = CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = lease(
        ExecutionPosture::Automatic,
        4,
        20,
        cancelled.token(),
        None,
        DeterminismContract::CanonicalBitwise,
    );
    assert!(matches!(map.run(Some(&cancelled_lease), |n, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*n)
    }), MapOutcome::Stopped { completed_prefix, boundary: Some(id), reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::Cancelled), .. }, .. }
    if completed_prefix.is_empty() && id == PartitionIdentity::new(1)));

    let exhausted = lease(
        ExecutionPosture::Automatic,
        4,
        2,
        CancellationToken::new(),
        None,
        DeterminismContract::CanonicalBitwise,
    );
    assert!(matches!(map.run(Some(&exhausted), |n, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*n)
    }), MapOutcome::Stopped { completed_prefix, boundary: Some(id), reason: MapStop::WorkExhausted { .. }, .. }
    if completed_prefix == vec![1, 2] && id == PartitionIdentity::new(3)));

    let expired = lease(
        ExecutionPosture::Serial,
        1,
        20,
        CancellationToken::new(),
        Some(Instant::now()),
        DeterminismContract::CanonicalBitwise,
    );
    assert!(matches!(map.run(Some(&expired), |n, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*n)
    }), MapOutcome::Stopped { completed_prefix, boundary: Some(id), reason: MapStop::Failure { cause: MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed), .. }, .. }
    if completed_prefix.is_empty() && id == PartitionIdentity::new(1)));

    let values = domains();
    let mut retained = decomposition();
    let full = retained
        .run(
            None,
            &domain_map(&values),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    let changed = [
        Subdomain {
            rhs: 6.0,
            ..values[0]
        },
        values[1],
    ];
    let stopped = retained.run(
        None,
        &domain_map(&changed),
        EDITIONS,
        interior,
        |_, context| -> Result<InterfaceSolution<f64, f64>, MapKernelFailure<()>> {
            context.checkpoint(1)?;
            Err(MapKernelFailure::Domain(()))
        },
        back,
    );
    assert!(
        matches!(stopped, Err(error) if matches!(error.cause, DecomposeFailure::Interface { .. }))
    );
    let reused = retained
        .run(
            None,
            &ExecutionMap::<Subdomain, u64>::try_from_declared_partitions(vec![], vec![]).unwrap(),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(
        reused
            .values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        full.values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
}
