//! A shared interruption before dispatch has one settlement at every width.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as authority;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial as Denial, WorthQueryManagedComputationInterruption as Interruption,
};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;
#[test]
fn interruption_present_at_dispatch_has_equal_outcome_charge_and_commits() {
    let mut serial = None;
    for width in [None, Some(1), Some(2), Some(4)] {
        let cancel = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancel.token(),
        );
        let policy = test_policy(
            std::num::NonZeroUsize::new(width.unwrap_or(1)).unwrap(),
            1 << 20,
        );
        let placement = width.map_or(RuntimeWorldExecutionPlacement::Serial(policy), |_| {
            RuntimeWorldExecutionPlacement::Leased {
                authority: authority(),
                policy,
            }
        });
        let execution = QueryRequestExecution::open(placement, &scope);
        let mut commits = Vec::new();
        let (outcome, report) = execution
            .execution_request()
            .unwrap()
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                let plans = (0..3)
                    .map(|rank| {
                        OwnerPlan::new(
                            vec![PreparedCompute::new(
                                0,
                                0,
                                |_| panic!("interrupted dispatch cannot compute"),
                                |_: &()| 0,
                            )],
                            move |_, _| Ok::<_, Denial>(rank),
                        )
                    })
                    .collect();
                cancel.cancel();
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap(),
                    plans,
                    &execution,
                );
                let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                    maximum_work_visits: Some(0),
                    maximum_preparation_bytes: None,
                });
                cursor.apply(
                    &mut admission,
                    |key, value, _| {
                        value?;
                        commits.push(key);
                        Ok(())
                    },
                    |_, denial| denial,
                )
            })
            .unwrap();
        assert_eq!(outcome, Err(Denial::Interrupted(Interruption::Cancelled)));
        assert_eq!(report.charged_work(), 0);
        assert!(commits.is_empty());
        let actual = (outcome, report.charged_work(), commits);
        assert_eq!(actual, *serial.get_or_insert_with(|| actual.clone()));
    }
}
