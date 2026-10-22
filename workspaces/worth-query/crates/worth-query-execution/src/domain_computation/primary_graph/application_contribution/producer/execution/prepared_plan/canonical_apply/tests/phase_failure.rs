//! Earlier owner completion failure wins over a later member panic.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn earlier_completion_failure_wins_over_later_compute_panic() {
    for width in [None, Some(1), Some(2), Some(4)] {
        let cancel = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancel.token(),
        );
        let policy = worth_foundational::ExecutionRequestPolicy::for_workers(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(width.unwrap_or(1)).unwrap(),
        );
        let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
            RuntimeWorldExecutionPlacement::Leased {
                authority: test_authority(),
                policy,
            }
        });
        let execution = QueryRequestExecution::open(placement, &scope);
        let mut completed = Vec::new();
        let plans = (0..3)
            .map(|rank| {
                let compute = if rank == 2 {
                    PreparedCompute::new::<(usize, bool)>(|_| panic!("later member fails"))
                } else {
                    PreparedCompute::declared_test_request(rank, false)
                };
                OwnerPlan::new(vec![compute], move |mut slots| {
                    let rank = slots.pop().unwrap().into_value::<(usize, bool)>().0;
                    if rank == 0 {
                        Err(rank)
                    } else {
                        Ok(rank)
                    }
                })
            })
            .collect();
        let mut result = None;
        let handoff = execution
            .execution_request()
            .unwrap()
            .run_native(|_| {
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap(),
                    plans,
                    &execution,
                );
                assert!(matches!(cursor.slots[2].1,
                    Err(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Panicked)),
                    "the later compute failure must actually be reached");

                result = Some(cursor.apply(|key, result| {
                            result?;
                            completed.push(key);
                            Ok(())
                        }, |key, _| key));
            });
        assert_eq!(
            handoff.unwrap_err(),
            worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped
            )
        );
        assert_eq!(
            result.expect("the native callback applied owner completion"),
            Err(0),
            "the earlier owner failure wins"
        );
        assert!(completed.is_empty(), "no failed member or suffix publishes");
    }
}
