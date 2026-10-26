//! Duplicate roots are input work and safe points, never fictional reads.
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::validation::custom_rule::structural_views::StructuralWalkEventForTest;
use crate::validation::custom_rule::{CustomInvariantInterruption, CustomPreparationBudget};
use worth_execution::CancellationSource;

#[derive(Clone, Copy)]
enum Case {
    Funded,
    WorkLimited,
    Cancelled,
}

#[test]
fn duplicate_raw_roots_spend_original_work_and_observe_interruption_before_deduplication() {
    let setup_host = crate::test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&setup_host);
    let graph = Graph::new(&setup, 2, &[(0, 1, KindId(2))]);
    let root = graph.entities[0];
    let seeds = vec![root; 1024];
    let baseline = graph
        .read(&setup, unbounded(), |view| {
            view.walk_outgoing_from(&[root], KindId(2))
        })
        .unwrap();
    for leased in [false, true] {
        for case in [Case::Funded, Case::WorkLimited, Case::Cancelled] {
            let cancellation = CancellationSource::new();
            let host = crate::test_execution_policy::serial_request()
                .with_cancellation(cancellation.token());
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(worth_execution::LeaseRequest {
                    policy: worth_foundational::ExecutionRequestPolicy::new(
                        worth_foundational::ExecutionPosture::Serial,
                        worth_foundational::DeterminismContract::CanonicalBitwise,
                        worth_foundational::ExecutionBudget::new(
                            std::num::NonZeroUsize::MIN,
                            None,
                            None,
                        ),
                    ),
                    cancellation: cancellation.token(),
                    deadline: None,
                })
                .unwrap();
            let execution = if leased {
                ExecutionRequest::leased(&lease)
            } else {
                ExecutionRequest::serial(&host)
            };
            let mut work = CustomInvariantWorkMeter::new(
                matches!(case, Case::WorkLimited).then(|| NonZeroU64::new(64).unwrap()),
            );
            if leased {
                work = work.with_lease(&CustomPreparationBudget::tracked(&lease).unwrap());
            }
            let seen = Arc::new(AtomicUsize::new(0));
            let observed = Arc::clone(&seen);
            let result = graph.read(&execution, work, |mut view| {
                view.observe_walk_for_test(Arc::new(move |event| {
                    if let StructuralWalkEventForTest::Seed(entity) = event {
                        assert_eq!(entity, root);
                        if observed.fetch_add(1, Ordering::SeqCst) == 31
                            && matches!(case, Case::Cancelled)
                        {
                            cancellation.cancel();
                        }
                    }
                }));
                view.walk_outgoing_from(&seeds, KindId(2))
            });
            match (case, result) {
                (Case::Funded, Ok(walk)) => {
                    assert_eq!(seen.load(Ordering::SeqCst), seeds.len());
                    assert_eq!(visits(&walk), visits(&baseline));
                    assert_eq!(walk.reads(), baseline.reads());
                    assert_eq!(walk.relations(), baseline.relations());
                }
                (Case::WorkLimited, Err(denial)) => {
                    assert_eq!(
                        denial.kind,
                        StructuralWalkDenialKind::Read(
                            crate::validation::custom_rule::StructuralReadError::WorkBudgetExceeded
                        )
                    );
                    assert_eq!(denial.reads, StructuralWalkReads::default());
                    assert!(
                        seen.load(Ordering::SeqCst) > 0
                            && seen.load(Ordering::SeqCst) < seeds.len()
                    );
                }
                (Case::Cancelled, Err(denial)) => {
                    assert_eq!(
                        denial.kind,
                        StructuralWalkDenialKind::Interrupted(
                            CustomInvariantInterruption::Cancelled
                        )
                    );
                    assert_eq!(denial.reads, StructuralWalkReads::default());
                    assert!(
                        seen.load(Ordering::SeqCst) > 0
                            && seen.load(Ordering::SeqCst) < seeds.len()
                    );
                }
                _ => panic!("the original input-stage control must decide the complete walk"),
            }
            assert_eq!(lease.charged_memory_bytes(), 0);
        }
    }
}
