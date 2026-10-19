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
                    vec![PreparedCompute::declared_test_request(0, 3, false)],
                    move |_, _| Err(failure),
                )
            } else {
                OwnerPlan::immediate(Err(failure))
            };
            let succeeding = OwnerPlan::new(
                vec![PreparedCompute::declared_test_request(1, 7, false)],
                |mut slots, _| Ok(slots.pop().unwrap().into_value::<(usize, bool)>().0),
            );
            let mut held = Vec::new();
            let mut applied = Vec::new();
            let (outcome, report) = execution
                .execution_request()
                .unwrap()
                .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                    let cursor = CanonicalApplyCursor::compute(
                        CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1]).unwrap(),
                        vec![failed, succeeding],
                        &execution,
                    );
                    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                        maximum_work_visits: Some(0),
                        maximum_preparation_bytes: None,
                    });
                    cursor.apply(
                        &mut admission,
                        |key, result, _| {
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
            assert_eq!(
                report.charged_work(),
                7 + if failure == Failure::Complete { 3 } else { 0 }
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
            .map(|(rank, work)| {
                OwnerPlan::new(
                    vec![PreparedCompute::declared_test_request(
                        rank,
                        work,
                        rank == 0,
                    )],
                    |mut slots, _| {
                        // The handler catches the transported denial and supplies its decision.
                        Ok::<_, ()>(slots.pop().unwrap().into_value::<(usize, bool)>().0)
                    },
                )
            })
            .collect();
        let mut applied = Vec::new();
        let (outcome, report) = execution
            .execution_request()
            .unwrap()
            .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
                let cursor = CanonicalApplyCursor::compute(
                    CanonicalUniqueVec::try_from_sorted_unique(vec![0, 1]).unwrap(),
                    plans,
                    &execution,
                );
                let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                    maximum_work_visits: Some(0),
                    maximum_preparation_bytes: None,
                });
                cursor.apply(
                    &mut admission,
                    |key, result, _| {
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
        assert_eq!(report.charged_work(), 3 + 7);
    }
}
