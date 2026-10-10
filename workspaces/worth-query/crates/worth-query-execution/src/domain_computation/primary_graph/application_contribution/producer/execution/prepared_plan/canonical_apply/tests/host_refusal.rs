//! Author-held handles return their cause to owner completion.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::test_execution_authority as test_authority;
use crate::domain_computation::primary_graph::{tests::fixture, WorthQueryAdvancementDenial};
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn a_refused_author_host_call_completes_on_the_owner_and_applies_only_its_prefix() {
    let application = Arc::new(fixture::installed_authorization_world(false).application);
    for workers in [1, 2, 4] {
        for product in [false, true] {
            let cancel = WorthQueryCancellationSource::new();
            let scope = WorthQueryRequestScope::new(
                Instant::now() + Duration::from_secs(60),
                cancel.token(),
            );
            let execution = QueryRequestExecution::open(
                RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy: test_policy(std::num::NonZeroUsize::new(workers).unwrap(), 1 << 24),
                },
                &scope,
            );
            let plans = (0_usize..3).map(|rank| {
                let application = application.clone();
                OwnerPlan::new(vec![PreparedCompute::new(0, 0, move |_| {
                    if rank == 1 {
                        if product {
                            application.product_runtime().with_advancement(|_| rank)
                        } else {
                            application.with_host_advancement(|_| rank)
                        }
                    } else { Ok(rank) }
                }, |_| 0, |_| false)], |mut slots, _| {
                    assert!(!crate::domain_computation::primary_graph::application_contribution::computation::worker_custody::active(), "completion stays on the owner");
                    slots.pop().unwrap().into_value::<Result<usize, WorthQueryAdvancementDenial>>()
                })
            }).collect();
            let mut applied = Vec::new();
            let (result, _) = execution
                .execution_request()
                .unwrap()
                .run(worth_execution::ExecutionWorkCeiling::new(u64::MAX), |_| {
                    let cursor = CanonicalApplyCursor::compute(
                        CanonicalUniqueVec::try_from_sorted_unique(vec![0_usize, 1, 2]).unwrap(),
                        plans,
                        &execution,
                    );
                    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
                        maximum_work_visits: 0,
                        maximum_preparation_bytes: 0,
                    });
                    cursor.apply(
                        &mut admission,
                        |key, value, _| {
                            assert_eq!(key, value);
                            applied.push(key);
                            Ok(())
                        },
                        |_, cause| cause,
                    )
                })
                .unwrap();
            assert_eq!(result, Err(WorthQueryAdvancementDenial::NestedOpening));
            assert_eq!(
                applied,
                vec![0],
                "the earlier member applies; the failure and suffix do not"
            );
        }
    }
}
