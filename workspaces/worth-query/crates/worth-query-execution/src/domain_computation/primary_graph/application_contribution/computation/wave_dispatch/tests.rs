//! Declared charges and canonical outcomes judge the one production dispatch.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    test_authority, test_policy,
};
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

fn run(
    width: Option<usize>,
    members: Vec<PreparedMember>,
) -> (
    Vec<Result<ComputedMember, WorthQueryAdvancementDenial>>,
    worth_foundational::ExecutionReport,
) {
    let cancel = WorthQueryCancellationSource::new();
    let scope =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), cancel.token());
    let policy = test_policy(NonZeroUsize::new(width.unwrap_or(1)).unwrap(), 1 << 24);
    let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
        RuntimeWorldExecutionPlacement::Leased {
            authority: test_authority(),
            policy,
        }
    });
    let request = QueryRequestExecution::open(placement, &scope);
    request
        .execution_request()
        .unwrap()
        .run(worth_execution::ExecutionWorkCeiling::new(u64::MAX), |_| {
            dispatch(&request, members)
        })
        .unwrap()
}
fn charged(rank: u64, units: u64, fails: bool) -> PreparedMember {
    PreparedMember {
        preparation_failed: false,
        requests: vec![PreparedCompute::new(
            0,
            0,
            move |member| {
                let (_, mut charge) = member.into_parts();
                charge.as_mut().unwrap()(units).unwrap();
                (rank, fails)
            },
            |_| 0,
            |result| result.1,
        )],
    }
}
#[test]
fn canonical_failure_settles_only_its_declared_compute_prefix_at_every_width() {
    const CHARGES: [u64; 4] = [3, 1, 4, 2];
    for failure in 0..4 {
        for width in [None, Some(1), Some(2), Some(4)] {
            let (settled, report) = run(
                width,
                CHARGES
                    .into_iter()
                    .enumerate()
                    .map(|(rank, units)| charged(rank as u64, units, rank == failure))
                    .collect(),
            );
            assert_eq!(
                settled.len(),
                failure + 1,
                "a suffix has no result accessor"
            );
            let actual: Vec<_> = settled
                .into_iter()
                .map(|slots| {
                    let mut slots = slots.unwrap().into_slots();
                    assert_eq!(slots.len(), 1);
                    slots.pop().unwrap().into_value::<(u64, bool)>()
                })
                .collect();
            assert_eq!(
                actual,
                (0..=failure)
                    .map(|rank| (rank as u64, rank == failure))
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                report.charged_work(),
                CHARGES[..=failure].iter().sum::<u64>()
            );
        }
    }
}
#[test]
fn independent_member_spans_compose_by_max_at_serial_and_leased_posture() {
    const CHARGES: [u64; 4] = [3, 1, 4, 2];
    for width in [None, Some(1), Some(2), Some(4)] {
        let (settled, report) = run(
            width,
            CHARGES
                .into_iter()
                .enumerate()
                .map(|(rank, units)| charged(rank as u64, units, false))
                .collect(),
        );
        assert_eq!(settled.len(), CHARGES.len());
        assert!(settled.iter().all(Result::is_ok));
        assert_eq!(report.charged_work(), CHARGES.iter().sum::<u64>());
        assert_eq!(report.charged_span(), *CHARGES.iter().max().unwrap());
    }
}
#[test]
fn a_panicking_compute_drops_its_suffix_and_clears_worker_custody() {
    let after = Arc::new(Mutex::new(false));
    let probe = after.clone();
    let panics = PreparedMember {
        preparation_failed: false,
        requests: vec![PreparedCompute::new::<()>(
            0,
            0,
            |_| panic!("declared failure"),
            |_| 0,
            |_| false,
        )],
    };
    let (settled, _) = run(
        Some(2),
        vec![charged(0, 1, false), panics, charged(2, 4, false)],
    );
    assert_eq!(settled.len(), 2);
    assert!(matches!(
        settled[1],
        Err(WorthQueryAdvancementDenial::Panicked)
    ));
    let (settled, _) = run(
        Some(2),
        vec![PreparedMember {
            preparation_failed: false,
            requests: vec![PreparedCompute::new(
                0,
                0,
                move |_| {
                    *probe.lock().unwrap() = true;
                },
                |_| 0,
                |_| false,
            )],
        }],
    );
    assert_eq!(settled.len(), 1);
    assert!(*after.lock().unwrap());
    assert!(!super::super::worker_custody::active());
}

#[test]
fn a_later_member_finishes_first_even_when_a_canonical_member_fails() {
    use std::sync::Condvar;
    for failure in [None, Some(0), Some(1), Some(2), Some(3)] {
        for workers in [2, 4] {
            let schedule = Arc::new((Mutex::new((Vec::new(), false)), Condvar::new()));
            let members = (0..4)
                .map(|rank| {
                    let schedule = schedule.clone();
                    PreparedMember {
                        preparation_failed: false,
                        requests: vec![PreparedCompute::new(
                            0,
                            0,
                            move |member| {
                                if rank == 0 {
                                    let (state, wake) = &*schedule;
                                    let (mut state, waited) = wake
                                        .wait_timeout_while(
                                            state.lock().unwrap(),
                                            Duration::from_secs(60),
                                            |state| !state.0.contains(&3),
                                        )
                                        .unwrap();
                                    // Failure watchdog only; a passing schedule wakes immediately.
                                    state.1 |= waited.timed_out() && !state.0.contains(&3);
                                }
                                let (_, mut charge) = member.into_parts();
                                charge.as_mut().unwrap()(rank as u64 + 1).unwrap();
                                let (state, wake) = &*schedule;
                                state.lock().unwrap().0.push(rank);
                                wake.notify_all();
                                (rank, failure == Some(rank))
                            },
                            |_| 0,
                            |result| result.1,
                        )],
                    }
                })
                .collect();
            let (settled, report) = run(Some(workers), members);
            let (finished, watchdog) = &*schedule.0.lock().unwrap();
            assert!(
                !watchdog,
                "member 0 could not overlap member 3: {finished:?}"
            );
            assert!(
                finished.iter().position(|rank| *rank == 3).unwrap()
                    < finished.iter().position(|rank| *rank == 0).unwrap(),
                "the constructed schedule must happen"
            );
            let boundary = failure.map_or(4, |rank| rank + 1);
            assert_eq!(settled.len(), boundary);
            let values: Vec<_> = settled
                .into_iter()
                .map(|slots| {
                    slots
                        .unwrap()
                        .into_slots()
                        .pop()
                        .unwrap()
                        .into_value::<(usize, bool)>()
                })
                .collect();
            assert_eq!(
                values,
                (0..boundary)
                    .map(|rank| (rank, failure == Some(rank)))
                    .collect::<Vec<_>>()
            );
            assert_eq!(report.charged_work(), (1..=boundary as u64).sum::<u64>());
        }
    }
}

mod exhaustion;
mod interruptions;
