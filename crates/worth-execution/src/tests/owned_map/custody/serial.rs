use super::*;
use crate::{ExecutionWorkCeiling, SerialRequest, WorkCeilingDenial};

#[test]
fn owned_map_serial_taking_keeps_input_custody() {
    let _lock = TEST_LOCK.lock().unwrap();
    for stop in [
        Stop::Complete,
        Stop::Domain,
        Stop::Panic,
        Stop::Cancelled,
        Stop::Work,
        Stop::Overflow,
        Stop::Capacity,
        Stop::Memory,
    ] {
        let budget = SerialMemoryBudget::from_policy(&request(1, BUDGET, u64::MAX).policy);
        let ledger = Ledger::Serial(&budget);
        let baseline = ledger.reserve(BASELINE).unwrap();
        let hold = ledger.reserve(input_bytes()).unwrap();
        let drops: Vec<_> = (0..ITEMS).map(|_| AtomicUsize::new(0)).collect();
        let custody_violations = AtomicUsize::new(0);
        let source = crate::CancellationSource::new();
        let request = SerialRequest {
            memory: Some(budget.clone()),
            cancellation: source.token(),
            deadline: None,
        };
        let ceiling = if matches!(stop, Stop::Work) {
            0
        } else {
            u64::MAX
        };
        let (outcome, _) = ExecutionWorkCeiling::new(ceiling)
            .run_serial(&request, || {
                let expected_charge = AtomicU64::new(BASELINE + input_bytes());
                let values =
                    tracked_inputs(&drops, &custody_violations, ledger, &expected_charge, None);
                if matches!(stop, Stop::Cancelled) {
                    source.cancel();
                }
                let extra = matches!(stop, Stop::Memory)
                    .then(|| ledger.reserve(BUDGET - ledger.charged()).unwrap());
                // The enclosing serial request declares one token and already
                // owns the physical ledger; the map declares one worker.
                let bytes = memory_model::bytes::<Tracked<'_>, ResultBytes, ()>(
                    ITEMS,
                    ITEMS as u64 * PAYLOAD_BYTES,
                    access_bytes(),
                    1,
                    1,
                    false,
                );
                expected_charge.store(
                    if extra.is_some() {
                        BUDGET
                    } else {
                        ledger.charged() - input_bytes() + bytes
                    },
                    Ordering::SeqCst,
                );
                let outcome = map(values)
                    .run_owned_taking(None, hold, |input, ctx| evaluate(input, ctx, stop));
                drop(extra);
                outcome
            })
            .unwrap();
        assert_stop(outcome, stop);
        assert!(drops
            .iter()
            .all(|counter| counter.load(Ordering::SeqCst) == 1));
        assert_eq!(custody_violations.load(Ordering::SeqCst), 0);
        assert_eq!(ledger.charged(), BASELINE);
        drop(baseline);
        assert_eq!(ledger.charged(), 0);
    }
}

#[test]
fn expired_serial_request_preserves_callers_input_hold() {
    let _lock = TEST_LOCK.lock().unwrap();
    let budget = SerialMemoryBudget::from_policy(&request(1, BUDGET, u64::MAX).policy);
    let ledger = Ledger::Serial(&budget);
    let baseline = ledger.reserve(BASELINE).unwrap();
    let mut hold = Some(ledger.reserve(input_bytes()).unwrap());
    let drops: Vec<_> = (0..ITEMS).map(|_| AtomicUsize::new(0)).collect();
    let violations = AtomicUsize::new(0);
    let expected = AtomicU64::new(BASELINE + input_bytes());
    let mut batch = Some(map(tracked_inputs(
        &drops,
        &violations,
        ledger,
        &expected,
        None,
    )));
    let request = SerialRequest {
        memory: Some(budget.clone()),
        deadline: Some(Instant::now() - Duration::from_secs(1)),
        cancellation: crate::CancellationSource::new().token(),
    };
    // An expired scope rejects before the map can accept the caller's hold.
    let result = ExecutionWorkCeiling::new(u64::MAX).run_serial(&request, || {
        batch
            .take()
            .unwrap()
            .run_owned_taking(None, hold.take().unwrap(), |input, ctx| {
                evaluate(input, ctx, Stop::Complete)
            })
    });
    assert!(matches!(
        result,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed))
    ));
    assert!(batch.is_some() && hold.is_some());
    assert_eq!(ledger.charged(), BASELINE + input_bytes());
    drop(batch.take());
    assert!(drops.iter().all(|count| count.load(Ordering::SeqCst) == 1));
    assert_eq!(violations.load(Ordering::SeqCst), 0);
    drop(hold.take());
    assert_eq!(ledger.charged(), BASELINE);
    drop(baseline);
    assert_eq!(ledger.charged(), 0);
}

#[test]
fn admitted_serial_owned_map_deadline_preserves_prefix_and_custody() {
    let _lock = TEST_LOCK.lock().unwrap();
    let watchdog = Instant::now() + Duration::from_secs(60);
    loop {
        let budget = SerialMemoryBudget::from_policy(&request(1, BUDGET, u64::MAX).policy);
        let ledger = Ledger::Serial(&budget);
        let baseline = ledger.reserve(BASELINE).unwrap();
        let mut hold = Some(ledger.reserve(input_bytes()).unwrap());
        let drops: Vec<_> = (0..ITEMS).map(|_| AtomicUsize::new(0)).collect();
        let violations = AtomicUsize::new(0);
        let entered = AtomicUsize::new(0);
        let deadline = Instant::now() + Duration::from_millis(100);
        let request = SerialRequest {
            memory: Some(budget.clone()),
            deadline: Some(deadline),
            cancellation: crate::CancellationSource::new().token(),
        };
        let result = ExecutionWorkCeiling::new(u64::MAX).run_serial(&request, || {
            let bytes = memory_model::bytes::<Tracked<'_>, ResultBytes, ()>(
                ITEMS,
                ITEMS as u64 * PAYLOAD_BYTES,
                access_bytes(),
                1,
                1,
                false,
            );
            let expected = AtomicU64::new(ledger.charged() - input_bytes() + bytes);
            map(tracked_inputs(&drops, &violations, ledger, &expected, None)).run_owned_taking(
                None,
                hold.take().unwrap(),
                |input, _| {
                    assert_eq!(entered.fetch_add(1, Ordering::SeqCst), 0);
                    assert_eq!(input.index.get(), 0);
                    assert_eq!(ledger.charged(), expected.load(Ordering::SeqCst));
                    // The first result completes after observed expiry, so the
                    // next item's safe point must stop regardless of scheduling.
                    while Instant::now() <= deadline {
                        std::thread::yield_now();
                    }
                    drop(input);
                    Ok::<_, MapKernelFailure<()>>(ResultBytes(Vec::new()))
                },
            )
        });
        assert_eq!(violations.load(Ordering::SeqCst), 0);
        if hold.is_some() {
            assert!(drops.iter().all(|count| count.load(Ordering::SeqCst) == 0));
            assert_eq!(ledger.charged(), BASELINE + input_bytes());
            drop(hold.take());
        } else {
            assert!(drops.iter().all(|count| count.load(Ordering::SeqCst) == 1));
        }
        assert_eq!(ledger.charged(), BASELINE);
        drop(baseline);
        assert_eq!(ledger.charged(), 0);
        let outcome = match result {
            Ok((outcome, _)) => Some(outcome),
            Err(WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)) => None,
            _ => panic!("serial scope must admit or reject its expired deadline"),
        };
        if let Some(outcome) = outcome {
            match outcome {
                MapOutcome::Stopped {
                    completed_prefix,
                    boundary,
                    reason: MapStop::Failure { identity, cause },
                    ..
                } => {
                    assert_eq!(
                        cause,
                        MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed)
                    );
                    let calls = entered.load(Ordering::SeqCst);
                    assert_eq!(completed_prefix.len(), calls);
                    assert!(completed_prefix.iter().all(|value| value.0.is_empty()));
                    let expected = worth_foundational::PartitionIdentity::new(calls as u64 + 1);
                    assert_eq!(boundary, Some(expected));
                    assert_eq!(identity, expected);
                    if calls == 1 {
                        return;
                    }
                    assert_eq!(calls, 0);
                }
                _ => panic!("admitted map must stop at its elapsed deadline"),
            }
        }
        // Retry expired setup rather than assume scope/kernel entry beats a
        // timer. Passing evidence requires the first kernel to have run.
        assert!(
            Instant::now() < watchdog,
            "deadline test never admitted its first kernel"
        );
    }
}
