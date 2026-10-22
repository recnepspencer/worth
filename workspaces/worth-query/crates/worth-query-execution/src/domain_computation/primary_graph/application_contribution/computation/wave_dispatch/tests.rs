//! Canonical outcomes and original controls judge the production member wave.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as test_authority;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

fn worker_policy(width: usize) -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::for_workers(
        worth_foundational::ExecutionPosture::Automatic,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        NonZeroUsize::new(width).unwrap(),
    )
}

fn run(
    width: Option<usize>,
    members: Vec<PreparedMember>,
) -> (
    Vec<Result<ComputedMember, WorthQueryAdvancementDenial>>,
    worth_foundational::ExecutionReport,
) {
    let (values, handoff) = run_handoff(width, members);
    let physical = handoff.expect("a successful wave keeps its native handoff");
    let posture = if physical.active_workers_high_watermark() > 1 {
        worth_foundational::ExecutionPosture::Automatic
    } else {
        worth_foundational::ExecutionPosture::Serial
    };
    (
        values.expect("the successful request entered its callback"),
        worth_foundational::ExecutionReport::native(posture, physical),
    )
}

fn run_stopped(
    width: Option<usize>,
    members: Vec<PreparedMember>,
) -> Vec<Result<ComputedMember, WorthQueryAdvancementDenial>> {
    let (values, handoff) = run_handoff(width, members);
    assert_eq!(
        handoff.unwrap_err(),
        worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::NestedStopped)
    );
    values.expect("the callback retained the wave's canonical outcomes before handoff")
}

fn run_handoff(
    width: Option<usize>,
    members: Vec<PreparedMember>,
) -> (
    Option<Vec<Result<ComputedMember, WorthQueryAdvancementDenial>>>,
    Result<worth_foundational::ExecutionPhysicalReport, worth_execution::WorkCeilingDenial>,
) {
    let cancel = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancel.token());
    let policy = worker_policy(width.unwrap_or(1));
    let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
        RuntimeWorldExecutionPlacement::Leased {
            authority: test_authority(),
            policy,
        }
    });
    let request = QueryRequestExecution::open(placement, &scope);
    let mut values = None;
    let handoff = request
        .execution_request()
        .unwrap()
        .run_native(|_| {
            values = Some(dispatch(&request, members));
        })
        .map(|((), physical)| physical);
    (values, handoff)
}
fn member_result(rank: u64, fails: bool) -> PreparedMember {
    PreparedMember {
        requests: vec![PreparedCompute::new(move |member| {
            let (_, poll) = member.into_parts();
            poll().unwrap();
            (rank, fails)
        })],
    }
}
#[test]
fn member_denials_remain_canonical_data_without_accounting_at_every_width() {
    const MEMBERS: [u64; 4] = [0, 1, 2, 3];
    for failure in 0..4 {
        for width in [None, Some(1), Some(2), Some(5)] {
            let (settled, report) = run(
                width,
                MEMBERS
                    .into_iter()
                    .map(|rank| member_result(rank, rank as usize == failure))
                    .collect(),
            );
            assert_eq!(
                settled.len(),
                MEMBERS.len(),
                "a member denial leaves every outcome for owner completion"
            );
            assert_eq!(report.accounted_cost(), None);
            assert!(report.physical().active_workers_high_watermark() >= 1);
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
                (0..MEMBERS.len())
                    .map(|rank| (rank as u64, rank == failure))
                    .collect::<Vec<_>>()
            );
        }
    }
}
#[test]
fn a_panicking_compute_drops_its_suffix_and_clears_worker_custody() {
    let after = Arc::new(Mutex::new(false));
    let probe = after.clone();
    let panics = PreparedMember {
        requests: vec![PreparedCompute::new::<()>(|_| panic!("declared failure"))],
    };
    let settled = run_stopped(
        Some(2),
        vec![member_result(0, false), panics, member_result(2, false)],
    );
    assert_eq!(settled.len(), 2);
    assert!(matches!(
        settled[1],
        Err(WorthQueryAdvancementDenial::Panicked)
    ));
    let (settled, _) = run(
        Some(2),
        vec![PreparedMember {
            requests: vec![PreparedCompute::new(move |_| {
                *probe.lock().unwrap() = true;
            })],
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
        for workers in [2, 5] {
            let schedule = Arc::new((Mutex::new((Vec::new(), false)), Condvar::new()));
            let members = (0..4)
                .map(|rank| {
                    let schedule = schedule.clone();
                    PreparedMember {
                        requests: vec![PreparedCompute::new(move |member| {
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
                            let (_, poll) = member.into_parts();
                            poll().unwrap();
                            let (state, wake) = &*schedule;
                            state.lock().unwrap().0.push(rank);
                            wake.notify_all();
                            (rank, failure == Some(rank))
                        })],
                    }
                })
                .collect();
            let (settled, report) = run(Some(workers), members);
            assert_eq!(report.accounted_cost(), None);
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
            let boundary = 4;
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
        }
    }
}

mod interruptions;
mod present_interruption;

mod nested_outcomes;

pub(super) mod seeded_dispatch;

mod diagnostic_charge;
mod overlap;
mod retained_bytes;
