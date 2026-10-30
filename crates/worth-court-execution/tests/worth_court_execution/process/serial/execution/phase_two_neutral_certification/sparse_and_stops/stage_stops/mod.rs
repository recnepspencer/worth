use {
    crate::phase_two_neutral_certification::isolated_context::integer_map,
    crate::phase_two_neutral_certification::isolated_context::lease,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::back,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::decomposition,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::domain_map,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::domains,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::interface,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::interior,
    crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::Subdomain,
    crate::phase_two_neutral_certification::isolated_context::EDITIONS,
    crate::phase_two_neutral_certification::isolated_context::TEST_LOCK, std::time::Instant,
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::DecomposeFailure, worth_execution::ExecutionMap,
    worth_execution::InterfaceSolution, worth_execution::MapKernelFailure,
    worth_execution::MapKernelStop, worth_execution::MapOutcome, worth_execution::MapStop,
    worth_foundational::DeterminismContract, worth_foundational::ExecutionPosture,
    worth_foundational::PartitionIdentity,
};

#[test]
fn stage_stops_keep_canonical_prefix_and_do_not_publish_partial_decomposition() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let map = integer_map(4);
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = lease(
        worth_foundational::ExecutionPosture::Automatic,
        4,
        20,
        cancelled.token(),
        None,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    assert!(matches!(map.run(Some(&cancelled_lease), |n, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*n)
    }), worth_execution::MapOutcome::Stopped { completed_prefix, boundary: Some(id), reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled), .. }, .. }
    if completed_prefix.is_empty() && id == worth_foundational::PartitionIdentity::new(1)));

    let exhausted = lease(
        worth_foundational::ExecutionPosture::Automatic,
        4,
        2,
        worth_execution::CancellationToken::new(),
        None,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    assert!(matches!(map.run(Some(&exhausted), |n, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*n)
    }), worth_execution::MapOutcome::Stopped { completed_prefix, boundary: Some(id), reason: worth_execution::MapStop::WorkExhausted { .. }, .. }
    if completed_prefix == vec![1, 2] && id == worth_foundational::PartitionIdentity::new(3)));

    let expired = lease(
        worth_foundational::ExecutionPosture::Serial,
        1,
        20,
        worth_execution::CancellationToken::new(),
        Some(std::time::Instant::now()),
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    assert!(matches!(map.run(Some(&expired), |n, context| {
        context.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>(*n)
    }), worth_execution::MapOutcome::Stopped { completed_prefix, boundary: Some(id), reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::DeadlineElapsed), .. }, .. }
    if completed_prefix.is_empty() && id == worth_foundational::PartitionIdentity::new(1)));

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
            Err(worth_execution::MapKernelFailure::Domain(()))
        },
        back,
    );
    assert!(
        matches!(stopped, Err(error) if matches!(error.cause, worth_execution::DecomposeFailure::Interface { .. }))
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
