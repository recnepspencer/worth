//! Owner completion failure preserves the canonical publication boundary.
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::canonical_apply::tests::*;
use worth_query_execution::facade::domain_computation::query_test_execution_authority_for_test as test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn completion_failure_publishes_no_member_or_suffix() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    const MEMBERS: [usize; 4] = [0, 1, 2, 3];
    const FAILURE: usize = 0;
    for width in [None, Some(1), Some(2), Some(5)] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
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
        let mut applied = Vec::new();
        let plans = MEMBERS
            .into_iter()
            .map(|rank| {
                OwnerPlan::new_for_test(
                    vec![PreparedCompute::declared_test_request_for_test(rank, false)],
                    move |mut slots| {
                        let (actual_rank, _) =
                            slots.pop().unwrap().into_value_for_test::<(usize, bool)>();
                        if actual_rank == FAILURE {
                            Err(actual_rank)
                        } else {
                            Ok(actual_rank)
                        }
                    },
                )
            })
            .collect();
        let (result, report) = execution
            .execution_request()
            .unwrap()
            .run_native(|_| {
                let cursor = CanonicalApplyCursor::compute_for_test(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2, 3]).unwrap(),
                    plans,
                    &execution,
                );

                cursor.apply_for_test(
                    |key, result| {
                        result?;
                        applied.push(key);
                        Ok(())
                    },
                    |key, _| key,
                )
            })
            .unwrap();
        assert_eq!(
            result,
            Err(FAILURE),
            "the canonical completion reports its own failure"
        );
        assert!(
            applied.is_empty(),
            "the failed first member and its suffix publish nothing"
        );
        assert!(report.active_workers_high_watermark() >= 1);
    }
}
