//! Earlier owner completion failure wins over a later member panic.
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::canonical_apply::tests::*;
use worth_query_execution::facade::domain_computation::query_test_execution_authority_for_test as test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn earlier_completion_failure_wins_over_later_compute_panic() {
    for width in [None, Some(1), Some(2), Some(4)] {
        let cancel = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
            cancel.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
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
        let execution = QueryRequestExecution::open_for_test(placement, &scope);
        let mut completed = Vec::new();
        let plans = (0..3)
            .map(|rank| {
                let compute = if rank == 2 {
                    PreparedCompute::new_for_test::<(usize, bool)>(|_| panic!("later member fails"))
                } else {
                    PreparedCompute::declared_test_request_for_test(rank, false)
                };
                OwnerPlan::new_for_test(vec![compute], move |mut slots| {
                    let rank = slots
                        .pop()
                        .unwrap()
                        .into_value_for_test::<(usize, bool)>()
                        .0;
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
                let cursor = CanonicalApplyCursor::compute_for_test(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap(),
                    plans,
                    &execution,
                );
                assert!(matches!((*cursor.slots_for_test())[2].1,
                    Err(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Panicked)),
                    "the later compute failure must actually be reached");

                result = Some(cursor.apply_for_test(|key, result| {
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
