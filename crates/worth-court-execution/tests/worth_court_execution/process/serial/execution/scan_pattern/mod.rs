#[allow(unused_imports)]
pub(crate) use {std::sync::atomic::AtomicUsize, worth_execution::CancellationSource, worth_execution::CancellationToken, worth_foundational::ExecutionPosture, worth_execution::ExecutionScan, worth_execution::LeaseDenial, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapStop, std::sync::atomic::Ordering, worth_foundational::PartitionIdentity, worth_execution::ScanOutcome, crate::scan_pattern::ordinary_context::TEST_LOCK, crate::scan_pattern::ordinary_context::lease, crate::scan_pattern::ordinary_context::scan};

#[test]
fn ordered_carry_and_prefixes_are_deterministic() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(3, worth_execution::CancellationToken::new());
    let outcome = scan().run(Some(&lease), 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        let next = *carry + *item;
        Ok::<_, MapKernelFailure<()>>((next, next))
    });
    assert!(
        matches!(&outcome, worth_execution::ScanOutcome::Complete { state: 6, prefixes, .. } if prefixes == &vec![1, 3, 6])
    );
    assert_eq!(outcome.report().charged_work(), 3);
    assert_eq!(outcome.report().charged_span(), 3);
    assert_eq!(
        outcome.report().resolved_posture(),
        worth_foundational::ExecutionPosture::Serial
    );
}

#[test]
fn work_ceiling_and_cancellation_stop_at_named_carry_boundaries() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let first_lease = lease(2, worth_execution::CancellationToken::new());
    let exhausted = scan().run(
        Some(&first_lease),
        0_u64,
        0,
        0,
        0,
        0,
        |carry, item, context| {
            context.checkpoint(1)?;
            let next = *carry + *item;
            Ok::<_, MapKernelFailure<()>>((next, next))
        },
    );
    assert!(matches!(exhausted, worth_execution::ScanOutcome::Stopped {
        completed_state: 3,
        completed_prefix,
        boundary: Some(identity),
        reason: worth_execution::MapStop::WorkExhausted { .. },
        ..
    } if completed_prefix == vec![1, 3] && identity == worth_foundational::PartitionIdentity::new(3)));

    let token = worth_execution::CancellationSource::new();
    let lease = lease(10, token.token());
    let cancelled = scan().run(Some(&lease), 0_u64, 0, 0, 0, 0, |carry, item, context| {
        context.checkpoint(1)?;
        if *item == 1 {
            token.cancel();
        }
        let next = *carry + *item;
        Ok::<_, MapKernelFailure<()>>((next, next))
    });
    assert!(matches!(cancelled, worth_execution::ScanOutcome::Stopped {
        completed_state: 1,
        completed_prefix,
        boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::Stop(worth_execution::MapKernelStop::Cancelled), .. },
        ..
    } if completed_prefix == vec![1] && identity == worth_foundational::PartitionIdentity::new(2)));
}

#[test]
fn oversized_domain_error_cannot_escape_reserved_capacity() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = lease(10, worth_execution::CancellationToken::new());
    let outcome = scan().run(Some(&lease), 0_u64, 0, 0, 8, 0, |_carry, _, context| {
        context.checkpoint(1)?;
        Err::<(u64, u64), _>(worth_execution::MapKernelFailure::Domain("x".repeat(1_048_576)))
    });
    assert!(matches!(&outcome, worth_execution::ScanOutcome::Stopped {
        completed_state: 0,
        completed_prefix,
        boundary: Some(identity),
        reason: worth_execution::MapStop::Failure { cause: worth_execution::MapKernelFailure::ResultCapacityExceeded, .. },
        ..
    } if completed_prefix.is_empty() && *identity == worth_foundational::PartitionIdentity::new(1)));
    assert!(outcome.report().physical().peak_charged_memory_bytes() <= 4096);
}

#[test]
fn identity_storage_is_admitted_before_scan_dispatch() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let ids: Vec<_> = (0..100_000).map(worth_foundational::PartitionIdentity::new).collect();
    let entries = ids.iter().copied().map(|identity| (identity, ())).collect();
    let scan = worth_execution::ExecutionScan::try_from_ordered(ids, entries).unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let lease = lease(10, worth_execution::CancellationToken::new());
    let outcome = scan.run(Some(&lease), 0_u64, 0, 0, 0, 0, |state, _, _| {
        calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok::<_, MapKernelFailure<()>>((*state, ()))
    });
    assert!(matches!(
        outcome,
        worth_execution::ScanOutcome::Stopped {
            reason: worth_execution::MapStop::Admission(worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity { admitted: 4096, .. }
            )),
            ..
        }
    ));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}
