//! Owner failure takes precedence over a later settled compute failure.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn earlier_completion_failure_does_not_refund_the_later_compute_boundary() {
    const CHARGES: [u64; 3] = [3, 1, 4];
    for width in [None, Some(1), Some(2), Some(4)] {
        let cancel = WorthQueryCancellationSource::new();
        let scope =
            WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), cancel.token());
        let policy = test_policy(
            std::num::NonZeroUsize::new(width.unwrap_or(1)).unwrap(),
            1 << 20,
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
                    move |mut slots, _| {
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
            .run(worth_execution::ExecutionWorkCeiling::new(u64::MAX), |_| {
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap(),
                    plans,
                    &execution,
                );
                let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                    maximum_work_visits: 0,
                    maximum_preparation_bytes: 0,
                });
                (
                    cursor.apply(
                        &mut admission,
                        |key, _, _| {
                            completed.push(key);
                            Ok(())
                        },
                        |key, _| key,
                    ),
                    (),
                )
            })
            .unwrap();
        assert_eq!(result, Err(0), "the earlier owner failure wins");
        assert!(completed.is_empty(), "no failed member or suffix publishes");
        assert_eq!(
            report.charged_work(),
            CHARGES.iter().sum::<u64>(),
            "settled compute is never refunded"
        );
    }
}
