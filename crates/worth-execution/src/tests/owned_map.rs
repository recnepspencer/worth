use bounded_overlap::Overlap;

use crate::{
    backend::BackendKind, ChargedBytes, ExecutionMap, MapKernelContext, MapKernelFailure,
    MapKernelStop, MapOutcome, MapPartition, MapStop,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use super::{authority, request, CancellationSource, LeaseDenial, TEST_LOCK};

mod admission_lifetime;
mod bounded_overlap;
mod custody;
mod dispatch_cancellation;
mod memory_model;
mod panic_payload;

const COUNT: usize = 4;

#[derive(Clone, Copy, Debug)]
enum Scenario {
    Success,
    Domain,
    Several,
    Panic,
    Cancelled,
    Deadline,
    Capacity,
    Memory,
    Work(u64),
}

fn map<T>(values: impl IntoIterator<Item = T>) -> ExecutionMap<T, u64> {
    let entries: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| MapPartition {
            identity: PartitionIdentity::new(index as u64 + 1),
            value,
            read_keys: Vec::new(),
            write_keys: vec![index as u64],
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(
        entries.iter().map(|p| p.identity).collect(),
        entries,
    )
    .unwrap()
}

fn seeded_values(seed: u64) -> Vec<u64> {
    (0..COUNT)
        .map(|index| {
            seed.wrapping_mul(6364136223846793005)
                .rotate_left(index as u32)
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
struct MeasuredValue {
    value: u64,
    payload: Vec<u8>,
}
impl ChargedBytes for MeasuredValue {
    fn additional_charged_bytes(&self) -> u64 {
        self.payload.capacity() as u64
    }
}

fn kernel(
    value: u64,
    index: usize,
    scenario: Scenario,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<MeasuredValue, MapKernelFailure<u64>> {
    context.checkpoint(1)?;
    match scenario {
        Scenario::Domain if index == 2 => Err(MapKernelFailure::Domain(index as u64)),
        Scenario::Several if index == 1 || index == 3 => {
            Err(MapKernelFailure::Domain(index as u64))
        }
        Scenario::Panic if index == 2 => panic!("contained owned-map kernel panic"),
        Scenario::Capacity if index == 2 => Ok(MeasuredValue {
            value,
            payload: vec![0],
        }),
        _ => Ok(MeasuredValue {
            value: value.wrapping_mul(17),
            payload: Vec::new(),
        }),
    }
}

type Observed = (
    Vec<u64>,
    Option<PartitionIdentity>,
    Option<MapStop<u64>>,
    ExecutionReport,
);

fn observe(outcome: MapOutcome<MeasuredValue, u64>) -> Observed {
    match outcome {
        MapOutcome::Complete { values, report } => (
            values.into_iter().map(|v| v.value).collect(),
            None,
            None,
            report,
        ),
        MapOutcome::Stopped {
            completed_prefix,
            boundary,
            reason,
            report,
        } => (
            completed_prefix.into_iter().map(|v| v.value).collect(),
            boundary,
            Some(reason),
            report,
        ),
    }
}

fn canonical(
    observed: &Observed,
) -> (
    Vec<u64>,
    Option<PartitionIdentity>,
    Option<WidthIndependentStop<'_>>,
    u64,
    u64,
    u64,
) {
    // Across widths compare values, prefix boundary, stop kind/level,
    // charged work/span and discarded work. Posture, fallback,
    // worker high-water, peak memory, queue width, steals and denial bytes are
    // compared only between input modes at the same width.
    // Only denial byte counts vary with the framework's declared worker width.
    let stop = observed.2.as_ref().map(|stop| match stop {
        MapStop::Admission(LeaseDenial::MemoryExhausted(denial)) => {
            WidthIndependentStop::Memory(denial.level)
        }
        other => WidthIndependentStop::Other(other),
    });
    (
        observed.0.clone(),
        observed.1,
        stop,
        observed.3.charged_work(),
        observed.3.charged_span(),
        observed.3.physical().discarded_in_flight_work(),
    )
}

#[derive(Debug, PartialEq, Eq)]
enum WidthIndependentStop<'a> {
    Memory(crate::MemoryLimitLevel),
    Other(&'a MapStop<u64>),
}

#[test]
fn owned_map_differential_stops_and_schedules() {
    let _lock = TEST_LOCK.lock().unwrap();
    for scenario in [
        Scenario::Success,
        Scenario::Domain,
        Scenario::Several,
        Scenario::Panic,
        Scenario::Cancelled,
        Scenario::Deadline,
        Scenario::Capacity,
        Scenario::Memory,
        Scenario::Work(0),
        Scenario::Work(1),
        Scenario::Work(2),
        Scenario::Work(3),
        Scenario::Work(4),
    ] {
        for seed in [1, 73, 9871] {
            let values = seeded_values(seed);
            let mut oracle = None;
            for taking in [false, true] {
                for (workers, backend) in [
                    (1, BackendKind::Serial),
                    (1, BackendKind::Native),
                    (4, BackendKind::Native),
                    (4, BackendKind::Perturbation(seed)),
                ] {
                    let source = CancellationSource::new();
                    let work = if let Scenario::Work(work) = scenario {
                        work
                    } else {
                        100
                    };
                    let memory = if matches!(scenario, Scenario::Memory) {
                        1
                    } else {
                        2_000
                    };
                    let mut req = request(workers, memory, work);
                    req.cancellation = source.token();
                    if matches!(scenario, Scenario::Cancelled) {
                        source.cancel();
                    }
                    if matches!(scenario, Scenario::Deadline) {
                        req.deadline =
                            Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
                    }
                    let holder = (taking && matches!(scenario, Scenario::Memory)).then(|| {
                        authority()
                            .request_lease(request(workers, 2_000, work))
                            .unwrap()
                    });
                    let lease = match &holder {
                        Some(parent) => parent.child(req).unwrap(),
                        None => authority().request_lease(req).unwrap(),
                    };
                    let barrier = Overlap::new(workers);
                    let hold = taking.then(|| {
                        holder
                            .as_ref()
                            .unwrap_or(&lease)
                            .reserve_memory((COUNT * std::mem::size_of::<u64>()) as u64)
                            .unwrap()
                    });
                    let borrowed = observe(map(values.clone()).run_with_backend_taking(
                        Some(&lease),
                        backend,
                        hold,
                        |value, context| {
                            barrier.wait();
                            kernel(
                                *value,
                                values.iter().position(|v| v == value).unwrap(),
                                scenario,
                                context,
                            )
                        },
                    ));
                    let barrier = Overlap::new(workers);
                    let hold = taking.then(|| {
                        holder
                            .as_ref()
                            .unwrap_or(&lease)
                            .reserve_memory((COUNT * std::mem::size_of::<u64>()) as u64)
                            .unwrap()
                    });
                    let owned = observe(map(values.clone()).run_owned_with_backend_taking(
                        Some(&lease),
                        backend,
                        hold,
                        |value, context| {
                            barrier.wait();
                            kernel(
                                value,
                                values.iter().position(|v| *v == value).unwrap(),
                                scenario,
                                context,
                            )
                        },
                    ));
                    // Same configuration compares the entire denial, bytes included.
                    assert_eq!(
                        borrowed, owned,
                        "mode drift: {scenario:?}, workers={workers}, seed={seed}"
                    );
                    assert_expected(&owned, scenario, &values);
                    if let Some(expected) = &oracle {
                        assert_eq!(
                            canonical(expected),
                            canonical(&owned),
                            "schedule drift: {scenario:?}"
                        );
                    } else {
                        oracle = Some(owned);
                    }
                }
            }
            let expected = oracle.unwrap();
            assert_expected(&expected, scenario, &values);
        }
    }
}

fn assert_expected(observed: &Observed, scenario: Scenario, values: &[u64]) {
    let stop_at = match scenario {
        Scenario::Success | Scenario::Work(4) => None,
        Scenario::Domain | Scenario::Panic | Scenario::Capacity => Some(2),
        Scenario::Several => Some(1),
        Scenario::Cancelled | Scenario::Deadline => Some(0),
        Scenario::Work(work) => Some(work as usize),
        Scenario::Memory => {
            assert!(matches!(
                observed.2,
                Some(MapStop::Admission(LeaseDenial::MemoryExhausted(_)))
            ));
            assert!(observed.0.is_empty());
            return;
        }
    };
    assert_eq!(
        observed.0,
        values[..stop_at.unwrap_or(COUNT)]
            .iter()
            .map(|v| v.wrapping_mul(17))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        observed.1,
        stop_at.map(|index| PartitionIdentity::new(index as u64 + 1))
    );
    let cause = match scenario {
        Scenario::Domain => Some(MapKernelFailure::Domain(2)),
        Scenario::Several => Some(MapKernelFailure::Domain(1)),
        Scenario::Panic => Some(MapKernelFailure::Panic),
        Scenario::Capacity => Some(MapKernelFailure::ResultCapacityExceeded),
        Scenario::Cancelled => Some(MapKernelFailure::Stop(MapKernelStop::Cancelled)),
        Scenario::Deadline => Some(MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed)),
        _ => None,
    };
    if let Some(cause) = cause {
        assert_eq!(
            observed.2,
            Some(MapStop::Failure {
                identity: observed.1.unwrap(),
                cause
            })
        );
    } else if let Some(index) = stop_at {
        assert_eq!(
            observed.2,
            Some(MapStop::WorkExhausted {
                identity: PartitionIdentity::new(index as u64 + 1)
            })
        );
    } else {
        assert!(observed.2.is_none());
    }
}

#[test]
fn owned_map_send_only_inputs_and_results_compile_and_run() {
    struct Local(std::cell::Cell<u64>);
    impl ChargedBytes for Local {
        fn additional_charged_bytes(&self) -> u64 {
            0
        }
    }
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority().request_lease(request(4, 2_000, 100)).unwrap();
    let result = map((0..COUNT as u64).map(|v| Local(std::cell::Cell::new(v)))).run_owned(
        Some(&lease),
        |value, context| {
            context.checkpoint(1)?;
            value.0.set(value.0.get() + 1);
            Ok::<_, MapKernelFailure<Local>>(value)
        },
    );
    match result {
        MapOutcome::Complete { values, .. } => assert_eq!(
            values.iter().map(|v| v.0.get()).collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        ),
        MapOutcome::Stopped { .. } => panic!("Send-only map must run"),
    }
}
