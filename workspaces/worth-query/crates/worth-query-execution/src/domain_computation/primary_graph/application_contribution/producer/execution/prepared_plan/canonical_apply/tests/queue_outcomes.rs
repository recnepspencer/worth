//! An owner can hold a queued failure while consuming its independent sibling.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::query_test_execution_authority_for_test as test_authority;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Failure {
    Prepare,
    Complete,
    Ready,
}

#[test]
fn a_queued_member_failure_does_not_withhold_its_independent_sibling() {
    for failure in [Failure::Prepare, Failure::Complete, Failure::Ready] {
        for width in [None, Some(1), Some(2), Some(4)] {
            let cancellation = WorthQueryCancellationSource::new();
            let scope = WorthQueryRequestScope::new(
                Some(Instant::now() + Duration::from_secs(60)),
                cancellation.token(),
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
            let failed = if failure == Failure::Complete {
                OwnerPlan::new(
                    vec![PreparedCompute::declared_test_request(0, false)],
                    move |_| Err(failure),
                )
            } else {
                OwnerPlan::immediate(Err(failure))
            };
            let succeeding = OwnerPlan::new(
                vec![PreparedCompute::declared_test_request(1, false)],
                |mut slots| Ok(slots.pop().unwrap().into_value::<(usize, bool)>().0),
            );
            let mut held = Vec::new();
            let mut applied = Vec::new();
            let (outcome, _) = execution
                .execution_request()
                .unwrap()
                .run_native(|_| {
                    let cursor = CanonicalApplyCursor::compute(
                        CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1]).unwrap(),
                        vec![failed, succeeding],
                        &execution,
                    );

                    cursor.apply(
                        |key, result| {
                            match result {
                                Ok(value) => {
                                    assert_eq!(key, value);
                                    applied.push(key);
                                }
                                Err(cause) => held.push((key, cause)),
                            }
                            Ok(())
                        },
                        |_, refusal| panic!("unexpected request stop: {refusal:?}"),
                    )
                })
                .unwrap();
            assert_eq!(outcome, Ok(()));
            assert_eq!(held, [(0, failure)]);
            assert_eq!(
                applied,
                [1],
                "the failed queued member does not strand its sibling"
            );
        }
    }
}

#[test]
fn a_completion_that_catches_a_compute_denial_applies_the_whole_wave() {
    for width in [None, Some(1), Some(2), Some(4)] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(),
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
        let plans = [3, 7]
            .into_iter()
            .enumerate()
            .map(|(rank, _)| {
                OwnerPlan::new(
                    vec![PreparedCompute::declared_test_request(rank, rank == 0)],
                    |mut slots| {
                        // The handler catches the transported denial and supplies its decision.
                        Ok::<_, ()>(slots.pop().unwrap().into_value::<(usize, bool)>().0)
                    },
                )
            })
            .collect();
        let mut applied = Vec::new();
        let (outcome, _) = execution
            .execution_request()
            .unwrap()
            .run_native(|_| {
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1]).unwrap(),
                    plans,
                    &execution,
                );

                cursor.apply(
                    |key, result| {
                        assert_eq!(key, result?);
                        applied.push(key);
                        Ok(())
                    },
                    |_, refusal| panic!("unexpected request stop: {refusal:?}"),
                )
            })
            .unwrap();
        assert_eq!(outcome, Ok(()));
        assert_eq!(applied, [0, 1]);
    }
}
