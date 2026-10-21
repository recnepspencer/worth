//! A settled wave stays charged; only owner application stops at the failure.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn completion_failure_charges_the_whole_dispatched_wave() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    // Each scalar is a declared kernel checkpoint, not a report measurement.
    const CHARGES: [u64; 4] = [3, 1, 4, 2];
    const FAILURE: usize = 0;
    let expected_work = CHARGES.iter().sum::<u64>();
    let expected_span = *CHARGES.iter().max().unwrap();
    let mut observations = Vec::new();
    for width in [None, Some(1), Some(2), Some(5)] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token());
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
        let mut applied = Vec::new();
        let plans = CHARGES
            .into_iter()
            .enumerate()
            .map(|(rank, units)| {
                OwnerPlan::new(
                    vec![PreparedCompute::declared_test_request(rank, units, false)],
                    move |mut slots| {
                        let (actual_rank, _) = slots.pop().unwrap().into_value::<(usize, bool)>();
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
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1, 2, 3]).unwrap(),
                    plans,
                    &execution,
                );

                cursor.apply(
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
        observations.push((width, report.charged_work(), report.charged_span()));
    }
    let expected: Vec<_> = [None, Some(1), Some(2), Some(5)]
        .into_iter()
        .map(|width| (width, expected_work, expected_span))
        .collect();
    let judge = |model| {
        assert_eq!(
            observations, model,
            "a failed advancement charges every member of its dispatched wave"
        )
    };
    judge(expected);
    let prefix = [None, Some(1), Some(2), Some(5)]
        .into_iter()
        .map(|width| {
            (
                width,
                CHARGES[..=FAILURE].iter().sum::<u64>(),
                *CHARGES[..=FAILURE].iter().max().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert!(
        std::panic::catch_unwind(|| judge(prefix)).is_err(),
        "the same law assertion rejects a prefix-only mutant model"
    );
}
