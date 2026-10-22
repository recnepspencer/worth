//! Interruption is caused at a witnessed member checkpoint, never inferred from a prefix.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption as Interruption;
use std::sync::Condvar;
const MEMBERS: usize = 4;
#[derive(Default)]
struct Schedule {
    completed: [bool; 4],
    reached: bool,
    fired: bool,
    watchdog: bool,
}
#[test]
fn cancellation_during_compute_has_a_witness_and_a_safe_canonical_prefix() {
    for selected in 0..4 {
        for width in [None, Some(1), Some(2), Some(5)] {
            let cancellation = WorthQueryCancellationSource::new();
            let schedule = Arc::new((Mutex::new(Schedule::default()), Condvar::new()));
            let scope = WorthQueryRequestScope::new(None, cancellation.token());
            let policy = worker_policy(width.unwrap_or(1));
            let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
                RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy,
                }
            });
            let root = QueryRequestExecution::open(placement, &scope);
            let members = (0..MEMBERS)
                .map(|rank| {
                    let schedule = schedule.clone();
                    let cancellation = cancellation.clone();
                    PreparedMember {
                        requests: vec![PreparedCompute::new(move |member| {
                            let (state, wake) = &*schedule;
                            if rank == selected {
                                let (mut held, waited) = wake
                                    .wait_timeout_while(
                                        state.lock().unwrap(),
                                        Duration::from_secs(60),
                                        |held| !held.completed[..rank].iter().all(|done| *done),
                                    )
                                    .unwrap();
                                held.watchdog |= waited.timed_out()
                                    && !held.completed[..rank].iter().all(|done| *done);
                                held.reached = !held.watchdog && member.interruption().is_none();
                                drop(held);
                                cancellation.cancel();
                                let mut state = state.lock().unwrap();
                                state.fired = true;
                                wake.notify_all();
                            } else if rank > selected {
                                let (mut state, waited) = wake
                                    .wait_timeout_while(
                                        state.lock().unwrap(),
                                        Duration::from_secs(60),
                                        |state| !state.fired,
                                    )
                                    .unwrap();
                                state.watchdog |= waited.timed_out() && !state.fired;
                            }
                            let (_, poll) = member.into_parts();
                            let result = poll().map(|()| rank);
                            if result.is_ok() {
                                state.lock().unwrap().completed[rank] = true;
                                wake.notify_all();
                            }
                            result
                        })],
                    }
                })
                .collect();
            let mut settled = None;
            let handoff = root.execution_request().unwrap().run_native(|_| {
                settled = Some(dispatch(&root, members));
            });
            assert_eq!(
                handoff.unwrap_err(),
                worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::Cancelled
                )
            );
            let settled = settled.expect("the callback reached its witnessed cancellation");
            let state = schedule.0.lock().unwrap();
            assert!(
                !state.watchdog,
                "rendezvous watchdog: width={width:?}, member={selected}"
            );
            assert!(state.reached && state.fired,
                    "the selected member never reached its interruption point: width={width:?}, member={selected}");
            assert!(state.completed[..selected].iter().all(|done| *done));
            let boundary = settled.len() - 1;
            if width.is_none() || width == Some(1) {
                assert_eq!(boundary, selected);
            } else {
                // An earlier member may still be leaving its checkpoint when
                // the shared request interrupts; it may become the boundary.
                assert!(boundary <= selected);
            }
            let expected = Interruption::Cancelled;
            assert!(
                matches!(&settled[boundary], Err(WorthQueryAdvancementDenial::Interrupted(kind)) if *kind == expected)
            );
            let values: Vec<_> = settled
                .into_iter()
                .take(boundary)
                .map(|result| {
                    result
                        .unwrap()
                        .into_slots()
                        .pop()
                        .unwrap()
                        .into_value::<Result<usize, worth_execution::MapKernelStop>>()
                        .unwrap()
                })
                .collect();
            assert_eq!(values, (0..boundary).collect::<Vec<_>>());
        }
    }
}
