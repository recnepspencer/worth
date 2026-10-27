use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::*;
use crate::validation::custom_rule::{CustomInvariantInterruption, CustomPreparationBudget};
use worth_execution::{CancellationSource, CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(
    memory: Option<u64>,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, memory, None),
            ),
            cancellation,
            deadline,
        })
        .unwrap()
}

#[test]
fn complete_walk_keeps_original_serial_and_leased_in_progress_stops() {
    let setup = crate::test_execution_policy::serial_request();
    let setup_execution = ExecutionRequest::serial(&setup);
    let graph = Graph::new(
        &setup_execution,
        6,
        &[
            (0, 1, KindId(2)),
            (1, 2, KindId(2)),
            (2, 3, KindId(2)),
            (3, 4, KindId(2)),
            (4, 5, KindId(2)),
        ],
    );
    for leased in [false, true] {
        for stop in [
            None,
            Some(CustomInvariantInterruption::Cancelled),
            Some(CustomInvariantInterruption::DeadlineElapsed),
        ] {
            let cancellation = CancellationSource::new();
            let seen = Arc::new(AtomicUsize::new(0));
            let deadline = (stop == Some(CustomInvariantInterruption::DeadlineElapsed))
                .then(|| Instant::now() + Duration::from_secs(1));
            let host = crate::test_execution_policy::serial_request()
                .with_cancellation(cancellation.token())
                .with_deadline(deadline);
            let lease = lease(None, cancellation.token(), deadline);
            let execution = if leased {
                ExecutionRequest::leased(&lease)
            } else {
                ExecutionRequest::serial(&host)
            };
            let work = if leased {
                unbounded().with_lease(&CustomPreparationBudget::tracked(&lease).unwrap())
            } else {
                unbounded()
            };
            let result = graph.read(&execution, work, |mut view| {
                let visits = Arc::clone(&seen);
                view.observe_walk_for_test(Arc::new(move |event| {
                    let crate::validation::custom_rule::structural_views::StructuralWalkEventForTest::Entity(_entity) = event else { return; };
                    if visits.fetch_add(1, Ordering::SeqCst) == 2 {
                        match stop {
                            Some(CustomInvariantInterruption::Cancelled) => cancellation.cancel(),
                            Some(CustomInvariantInterruption::DeadlineElapsed) => {
                                let deadline = deadline.unwrap();
                                std::thread::sleep(
                                    deadline.saturating_duration_since(Instant::now())
                                        + Duration::from_millis(1),
                                );
                            }
                            None => {}
                        }
                    }
                }));
                view.walk_outgoing_from(&[graph.entities[0]], KindId(2))
            });
            match stop {
                Some(stop) => {
                    let denial = result.unwrap_err();
                    assert_eq!(denial.kind, StructuralWalkDenialKind::Interrupted(stop));
                    assert!(
                        denial.reads.entity_records > 0
                            && denial.reads.entity_records < graph.entities.len()
                    );
                    assert!(denial.reads.adjacency_lists > 0);
                }
                None => {
                    assert_eq!(result.unwrap().steps().len(), graph.entities.len());
                    assert_eq!(seen.load(Ordering::SeqCst), graph.entities.len());
                }
            }
            assert_eq!(
                lease.charged_memory_bytes(),
                0,
                "no native payer escapes the invocation"
            );
        }
    }
}

#[test]
fn complete_walk_work_and_memory_refuse_without_a_partial_result() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let graph = Graph::new(
        &execution,
        5,
        &[
            (0, 1, KindId(2)),
            (1, 2, KindId(2)),
            (2, 3, KindId(2)),
            (3, 4, KindId(2)),
        ],
    );
    let short = CustomInvariantWorkMeter::new(NonZeroU64::new(8));
    let denial = graph
        .read(&execution, short, |view| {
            view.walk_outgoing_from(&[graph.entities[0]], KindId(2))
        })
        .unwrap_err();
    assert_eq!(
        denial.kind,
        StructuralWalkDenialKind::Read(
            crate::validation::custom_rule::StructuralReadError::WorkBudgetExceeded
        )
    );
    assert!(denial.reads.entity_records > 0 && denial.reads.entity_records < graph.entities.len());
    for memory in [2048, 1024 * 1024] {
        let lease = lease(Some(memory), CancellationToken::new(), None);
        let budget = CustomPreparationBudget::tracked(&lease).unwrap();
        let work = unbounded().with_lease(&budget);
        let request = ExecutionRequest::leased(&lease);
        let result = graph.read(&request, work, |view| {
            view.walk_outgoing_from(&[graph.entities[0]], KindId(2))
        });
        if memory == 2048 {
            let denial = result.unwrap_err();
            assert!(matches!(
                denial.kind,
                StructuralWalkDenialKind::MemoryAdmission(
                    worth_execution::MemoryLimitDenial::Capacity {
                        level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
                        admitted: 2048,
                        ..
                    }
                )
            ));
            assert!(denial.reads.entity_records < graph.entities.len());
        } else {
            assert_eq!(result.unwrap().steps().len(), graph.entities.len());
        }
        assert!(lease.charged_memory_bytes() > 0);
        drop(budget);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn retained_complete_result_keeps_its_original_payer_until_drop() {
    let host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let graph = Graph::new(&setup, 3, &[(0, 1, KindId(2)), (1, 2, KindId(2))]);
    let memory = 64 * 1024;
    let lease = lease(Some(memory), CancellationToken::new(), None);
    let budget = CustomPreparationBudget::tracked(&lease).unwrap();
    let result = graph
        .read(
            &ExecutionRequest::leased(&lease),
            unbounded().with_lease(&budget),
            |view| view.walk_outgoing_from(&[graph.entities[0]], KindId(2)),
        )
        .unwrap();
    drop(budget);
    assert_eq!(result.steps().len(), 3);
    assert!(
        lease.reserve_memory(memory).is_err(),
        "live result still holds the original payer"
    );
    drop(result);
    let competing = lease
        .reserve_memory(memory)
        .expect("released result leaves the same original lineage funded");
    drop(competing);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn complete_walk_counts_acquired_metadata_even_when_access_is_denied() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let graph = Graph::new(&execution, 2, &[(0, 1, KindId(2))]);
    let observation =
        InvariantObservation::committed(graph.runtime.storage_access().current_edition());
    for denied_relation in [false, true] {
        let state = InvariantStateView::new(
            observation.committed_partition_access(),
            graph.runtime.current_version_id(),
        );
        let access = CustomInvariantAccessContract {
            read_relation_kinds: if denied_relation {
                vec![]
            } else {
                vec![KindId(2)]
            },
            ..CustomInvariantAccessContract::default()
        };
        let view = StructuralRelationView::new(state, execution, unbounded(), Arc::new(access));
        let denial = view
            .walk_outgoing_from(&[graph.entities[0]], KindId(2))
            .unwrap_err();
        assert_eq!(
            denial.kind,
            StructuralWalkDenialKind::Read(
                crate::validation::custom_rule::StructuralReadError::OutsideDeclaredAccess
            )
        );
        assert_eq!(
            denial.reads.entity_records,
            usize::from(!denied_relation),
            "relation admission precedes metadata; entity admission follows acquisition"
        );
        assert_eq!(denial.reads.adjacency_lists, 0);
        assert_eq!(denial.reads.relation_records, 0);
    }
}
