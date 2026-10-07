use std::{
    cell::Cell,
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use super::{authority, map, request, ChargedBytes, MapKernelStop, MapStop, TEST_LOCK};
use crate::{
    ExecutionMemoryReservation, ExecutionResourceLease, MapKernelFailure, MapOutcome,
    SerialMemoryBudget,
};

mod serial;

use super::{memory_model, Overlap};

const BUDGET: u64 = 2_000;
const BASELINE: u64 = 17;
const ITEMS: usize = 2;
const PAYLOAD_BYTES: u64 = 32;

#[derive(Clone, Copy, Debug)]
enum Stop {
    Complete,
    Domain,
    Panic,
    Cancelled,
    Deadline,
    Work,
    Overflow,
    Capacity,
    Memory,
    Unrelated,
    Nested,
}

#[derive(Clone, Copy)]
enum Ledger<'a> {
    Lease(&'a ExecutionResourceLease<'a>),
    Serial(&'a SerialMemoryBudget),
}

impl Ledger<'_> {
    fn reserve(self, bytes: u64) -> Result<ExecutionMemoryReservation, crate::MemoryLimitDenial> {
        match self {
            Self::Lease(lease) => lease.reserve_memory(bytes),
            Self::Serial(budget) => budget.reserve(bytes),
        }
    }

    fn charged(self) -> u64 {
        match self.reserve(BUDGET) {
            Ok(held) => {
                drop(held);
                0
            }
            Err(denial) => BUDGET - denial.admitted,
        }
    }
}

struct Tracked<'a> {
    index: Cell<usize>,
    drops: &'a [AtomicUsize],
    custody_violations: &'a AtomicUsize,
    ledger: Ledger<'a>,
    expected_charge: &'a AtomicU64,
    payload: Vec<u8>,
    overlap: Option<&'a Overlap>,
}

impl ChargedBytes for Tracked<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        self.payload.capacity() as u64
    }
}

impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        if self.ledger.charged() != self.expected_charge.load(Ordering::SeqCst) {
            self.custody_violations.fetch_add(1, Ordering::SeqCst);
        }
        self.drops[self.index.get()].fetch_add(1, Ordering::SeqCst);
    }
}

struct ResultBytes(Vec<u8>);
impl ChargedBytes for ResultBytes {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64
    }
}

fn input_bytes() -> u64 {
    ITEMS as u64 * (std::mem::size_of::<Tracked<'_>>() as u64 + PAYLOAD_BYTES)
}

fn tracked_inputs<'a>(
    drops: &'a [AtomicUsize],
    custody_violations: &'a AtomicUsize,
    ledger: Ledger<'a>,
    expected_charge: &'a AtomicU64,
    overlap: Option<&'a Overlap>,
) -> Vec<Tracked<'a>> {
    (0..ITEMS)
        .map(|index| Tracked {
            index: Cell::new(index),
            drops,
            custody_violations,
            ledger,
            expected_charge,
            overlap,
            payload: vec![0; PAYLOAD_BYTES as usize],
        })
        .collect()
}

fn evaluate(
    input: Tracked<'_>,
    context: &mut crate::MapKernelContext<'_, '_>,
    stop: Stop,
) -> Result<ResultBytes, MapKernelFailure<()>> {
    let overlap = input.overlap;
    if let Some(overlap) = overlap {
        overlap.wait();
    }
    // Keep every worker in its kernel through every ledger read and input Drop.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        evaluate_input(input, context, stop)
    }));
    if let Some(overlap) = overlap {
        overlap.wait();
    }
    match result {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

fn evaluate_input(
    input: Tracked<'_>,
    context: &mut crate::MapKernelContext<'_, '_>,
    stop: Stop,
) -> Result<ResultBytes, MapKernelFailure<()>> {
    assert_eq!(
        input.ledger.charged(),
        input.expected_charge.load(Ordering::SeqCst),
        "worker has exactly the declared reservation"
    );
    match stop {
        Stop::Panic => panic!("owned input unwinds inside kernel"),
        Stop::Domain => return Err(MapKernelFailure::Domain(())),
        Stop::Capacity => return Ok(ResultBytes(vec![0])),
        Stop::Overflow => {
            context.checkpoint(u64::MAX)?;
            context.checkpoint(1)?;
        }
        Stop::Nested => {
            let _ = map([0_u64]).run(None, |value, _| Ok::<_, MapKernelFailure<()>>(*value));
            // A stopped nested scope propagates through the active kernel meter.
        }
        _ => {
            context.checkpoint(1)?;
        }
    }
    Ok(ResultBytes(Vec::new()))
}

fn assert_stop(outcome: MapOutcome<ResultBytes, ()>, stop: Stop) {
    let expected = match stop {
        Stop::Complete => {
            assert!(matches!(outcome, MapOutcome::Complete { .. }));
            return;
        }
        Stop::Domain => MapKernelFailure::Domain(()),
        Stop::Panic => MapKernelFailure::Panic,
        Stop::Cancelled => MapKernelFailure::Stop(MapKernelStop::Cancelled),
        Stop::Deadline => MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed),
        Stop::Overflow => MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow),
        Stop::Nested => MapKernelFailure::Stop(MapKernelStop::NestedStopped),
        Stop::Capacity => MapKernelFailure::ResultCapacityExceeded,
        Stop::Memory | Stop::Unrelated => {
            match outcome {
                MapOutcome::Stopped {
                    reason: MapStop::Admission(denial),
                    ..
                } => match stop {
                    Stop::Memory => {
                        assert!(matches!(denial, crate::LeaseDenial::MemoryExhausted(_)))
                    }
                    _ => assert_eq!(denial, crate::LeaseDenial::UnrelatedNestedLease),
                },
                _ => panic!("must refuse admission"),
            }
            return;
        }
        Stop::Work => {
            assert!(matches!(
                outcome,
                MapOutcome::Stopped {
                    reason: MapStop::WorkExhausted { .. },
                    ..
                }
            ));
            return;
        }
    };
    match outcome {
        MapOutcome::Stopped {
            reason: MapStop::Failure { cause, .. },
            ..
        } => assert_eq!(cause, expected),
        _ => panic!("expected kernel stop: {stop:?}"),
    }
}

#[test]
fn owned_map_drops_and_leased_custody_on_every_stop() {
    let _lock = TEST_LOCK.lock().unwrap();
    for workers in [1, 2] {
        for stop in [
            Stop::Complete,
            Stop::Domain,
            Stop::Panic,
            Stop::Cancelled,
            Stop::Deadline,
            Stop::Work,
            Stop::Overflow,
            Stop::Capacity,
            Stop::Memory,
            Stop::Unrelated,
            Stop::Nested,
        ] {
            let parent = authority()
                .request_lease(request(workers, BUDGET, u64::MAX))
                .unwrap();
            let ledger = Ledger::Lease(&parent);
            let baseline = ledger.reserve(BASELINE).unwrap();
            let hold = ledger.reserve(input_bytes()).unwrap();
            let drops: Vec<_> = (0..ITEMS).map(|_| AtomicUsize::new(0)).collect();
            let custody_violations = AtomicUsize::new(0);
            let expected_charge = AtomicU64::new(BASELINE + input_bytes());
            let overlap = Overlap::new(workers);
            let active = !matches!(
                stop,
                Stop::Cancelled | Stop::Deadline | Stop::Memory | Stop::Unrelated
            );
            let values = tracked_inputs(
                &drops,
                &custody_violations,
                ledger,
                &expected_charge,
                active.then_some(&overlap),
            );
            let source = crate::CancellationSource::new();
            let mut req = request(
                workers,
                if matches!(stop, Stop::Memory) {
                    input_bytes()
                } else {
                    BUDGET
                },
                if matches!(stop, Stop::Work) {
                    0
                } else {
                    u64::MAX
                },
            );
            req.cancellation = source.token();
            if matches!(stop, Stop::Cancelled) {
                source.cancel();
            }
            if matches!(stop, Stop::Deadline) {
                req.deadline = Some(Instant::now() - Duration::from_secs(1));
            }
            let child = parent.child(req).unwrap();
            if !matches!(stop, Stop::Memory | Stop::Unrelated) {
                // Parent + child declare two cancellation tokens; a root run
                // owns its physical ledger and reserves the declared worker width.
                let bytes = memory_model::bytes::<Tracked<'_>, ResultBytes, ()>(
                    ITEMS,
                    ITEMS as u64 * PAYLOAD_BYTES,
                    access_bytes(),
                    2,
                    workers,
                    true,
                );
                expected_charge.store(BASELINE + bytes, Ordering::SeqCst);
            }
            let outcome = if matches!(stop, Stop::Unrelated) {
                map(values).run_owned_taking(None, hold, |input, ctx| evaluate(input, ctx, stop))
            } else {
                map(values)
                    .run_owned_taking(Some(&child), hold, |input, ctx| evaluate(input, ctx, stop))
            };
            assert_stop(outcome, stop);
            assert!(
                drops
                    .iter()
                    .all(|counter| counter.load(Ordering::SeqCst) == 1),
                "live input after {stop:?}"
            );
            assert_eq!(
                custody_violations.load(Ordering::SeqCst),
                0,
                "input hold released before destruction: {stop:?}"
            );
            assert_eq!(
                ledger.charged(),
                BASELINE,
                "run must return to starting balance"
            );
            drop(baseline);
            assert_eq!(ledger.charged(), 0);
        }
    }
}

// The declared map has one u64 write key and no read keys per input.
fn access_bytes() -> u64 {
    (ITEMS
        * (std::mem::size_of::<Vec<u64>>()
            + std::mem::size_of::<(worth_foundational::PartitionIdentity, Vec<u64>)>()
            + std::mem::size_of::<u64>())) as u64
}
