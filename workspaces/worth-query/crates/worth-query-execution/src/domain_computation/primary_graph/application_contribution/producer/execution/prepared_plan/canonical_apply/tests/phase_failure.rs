//! Owner failure takes precedence over a later request stop without refund.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn earlier_completion_failure_does_not_refund_the_later_request_boundary() {
    const CHARGES: [u64; 3] = [3, 1, 4];
    for width in [None, Some(1), Some(2), Some(4)] {
        let cancel = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancel.token(),
        );
        let policy = test_policy(
            std::num::NonZeroUsize::new(width.unwrap_or(1)).unwrap(),
            None,
        );
        let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
            RuntimeWorldExecutionPlacement::Leased {
                authority: test_authority(),
                policy,
            }
        });
        let execution = QueryRequestExecution::open(placement, &scope);
        let mut completed = Vec::new();
        let plans = CHARGES
            .into_iter()
            .enumerate()
            .map(|(rank, units)| {
                OwnerPlan::new(
                    vec![PreparedCompute::declared_test_request(
                        rank,
                        units,
                        rank == 2,
                    )],
                    move |mut slots| {
                        let rank = slots.pop().unwrap().into_value::<(usize, bool)>().0;
                        if rank == 0 {
                            Err(rank)
                        } else {
                            Ok(rank)
                        }
                    },
                )
            })
            .collect();
        let ((result, ()), report) = execution
            .execution_request()
            .unwrap()
            .run(worth_execution::ExecutionWorkCeiling::new(Some(CHARGES[..2].iter().sum())), |_| {
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap(),
                    plans,
                    &execution,
                );
                assert!(matches!(cursor.slots[2].1,
                    Err(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                        crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted
                    ))), "the later root stops on the request's refused charge");

                (
                    cursor.apply(|key, result| {
                            result?;
                            completed.push(key);
                            Ok(())
                        }, |key, _| key),
                    (),
                )
            })
            .unwrap();
        assert_eq!(result, Err(0), "the earlier owner failure wins");
        assert!(completed.is_empty(), "no failed member or suffix publishes");
    }
}
