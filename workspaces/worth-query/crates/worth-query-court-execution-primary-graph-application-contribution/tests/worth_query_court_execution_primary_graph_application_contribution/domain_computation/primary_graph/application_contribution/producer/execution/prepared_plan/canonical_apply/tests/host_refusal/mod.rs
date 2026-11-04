//! Author-held handles return their cause to owner completion.
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::canonical_apply::tests::*;
use worth_query_execution::facade::domain_computation::query_test_execution_authority_for_test as test_authority;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_runtime_world::facade::RuntimeWorldExecutionPlacement;

#[test]
fn a_refused_author_host_call_completes_on_the_owner_and_applies_only_its_prefix() {
    let application = Arc::new(worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(false, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()).application);
    for workers in [1, 2, 4] {
        for product in [false, true] {
            let cancel = WorthQueryCancellationSource::new();
            let scope = WorthQueryRequestScope::new(
                Some(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(60)),
                cancel.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
            let execution = QueryRequestExecution::open_for_test(
                RuntimeWorldExecutionPlacement::Leased {
                    authority: test_authority(),
                    policy: test_policy(std::num::NonZeroUsize::new(workers).unwrap(), None),
                },
                &scope,
            );
            let plans = (0_usize..3).map(|rank| {
                let application = application.clone();
                OwnerPlan::new_for_test(vec![PreparedCompute::new_for_test( move |_| {
                    if rank == 1 {
                        if product {
                            application.product_runtime().with_advancement(|_| rank)
                        } else {
                            application.with_host_advancement(|_| rank)
                        }
                    } else { Ok(rank) }
                })], |mut slots| {
                    assert!(!worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::computation::worker_custody::active(), "completion stays on the owner");
                    slots.pop().unwrap().into_value_for_test::<Result<usize, WorthQueryAdvancementDenial>>()
                })
            }).collect();
            let mut applied = Vec::new();
            let (result, _) = execution
                .execution_request()
                .unwrap()
                .run_native(|_| {
                    let cursor = CanonicalApplyCursor::compute_for_test(
                        CanonicalUniqueVec::try_from_sorted_unique(vec![0_usize, 1, 2]).unwrap(),
                        plans,
                        &execution,
                    );

                    cursor.apply_for_test(
                        |key, value| {
                            let value = value?;
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
