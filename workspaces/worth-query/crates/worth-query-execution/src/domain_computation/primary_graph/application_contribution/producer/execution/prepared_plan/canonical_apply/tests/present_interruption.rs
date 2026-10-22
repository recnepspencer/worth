//! A shared interruption before dispatch has one settlement at every width.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as authority;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption as Interruption;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;
#[test]
fn interruption_present_at_dispatch_has_equal_original_stop_and_commits() {
    let mut serial = None;
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
                authority: authority(),
                policy,
            }
        });
        let execution = QueryRequestExecution::open(placement, &scope);
        let mut commits = Vec::new();
        let mut outcome = None;
        let handoff = execution.execution_request().unwrap().run_native(|_| {
            let plans = (0..3)
                .map(|rank| {
                    OwnerPlan::new(
                        vec![PreparedCompute::new(|_| {
                            panic!("interrupted dispatch cannot compute")
                        })],
                        move |_| Ok::<_, Denial>(rank),
                    )
                })
                .collect();
            cancel.cancel();
            let cursor = CanonicalApplyCursor::compute(
                CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2]).unwrap(),
                plans,
                &execution,
            );

            outcome = Some(cursor.apply(
                |key, value| {
                    value?;
                    commits.push(key);
                    Ok(())
                },
                |_, denial| denial,
            ));
        });
        assert_eq!(
            handoff.unwrap_err(),
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        );
        let outcome = outcome.expect("the owner observes its original cancelled cursor");
        assert_eq!(outcome, Err(Denial::Interrupted(Interruption::Cancelled)));

        assert!(commits.is_empty());
        let actual = (outcome, commits);

        assert_eq!(actual, *serial.get_or_insert_with(|| actual.clone()));
    }
}
