//! Cleanup issuance never decides the release order of genuine occurrences.
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::CleanupScope;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::WorthQueryProductBranchOwnerCleanupRegistry;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::WorthQueryRetiredProductOccurrence;
use worth_runtime_world::facade::*;

#[test]
fn retirement_releases_in_branch_then_incarnation_order() {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let root = runtime
        .admit_product_branch_for_test(runtime.default_branch_for_test())
        .unwrap();
    let child = |name| {
        let intent = ProductBranchCreationIntent::from_source(
            name,
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = runtime
            .create_product_branch_for_test(
                execution,
                &root,
                None,
                intent,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("a genuine child occurrence is created");
        };
        let occurrence = WorthQueryRetiredProductOccurrence::new_for_test(
            child.branch_identity().clone(),
            child.lifecycle_incarnation(),
        );
        let observed = runtime
            .admit_product_occurrence_for_test(child.lifecycle_incarnation())
            .unwrap();
        let report = runtime.retire_product_branch_for_test(&observed).unwrap();
        (occurrence, report)
    };
    let (zulu, zulu_report) = child("zulu");
    let (alpha_first, first_report) = child("alpha");
    let (alpha_second, second_report) = child("alpha");
    assert_eq!(alpha_first.branch(), alpha_second.branch());
    assert!(alpha_first.incarnation() < alpha_second.incarnation());
    let registry = WorthQueryProductBranchOwnerCleanupRegistry::new_for_test(Some(3));
    for (occurrence, report) in [
        (zulu.clone(), zulu_report),
        (alpha_second.clone(), second_report),
        (alpha_first.clone(), first_report),
    ] {
        registry
            .reserve_for_test(CleanupScope::ApplicationRetirement(occurrence))
            .unwrap()
            .install_retirement(report);
    }
    let actual = registry
        .pending_application_retired_product_occurrences_for_test()
        .into_iter()
        .map(|occurrence| (occurrence.branch().clone(), occurrence.incarnation()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (alpha_first.branch().clone(), alpha_first.incarnation()),
            (alpha_second.branch().clone(), alpha_second.incarnation()),
            (zulu.branch().clone(), zulu.incarnation()),
        ]
    );
}

#[test]
fn absent_cleanup_policy_keeps_reservations_until_drop() {
    let registry = worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::WorthQueryProductBranchOwnerCleanupRegistry::new_for_test(None);
    let held: Vec<_> = (0..600)
        .map(|_| registry.reserve_for_test(worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::CleanupScope::Application).unwrap())
        .collect();
    assert_eq!(worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::lock(&(*registry.state_for_test())).reserved, held.len());
    drop(held);
    assert_eq!(worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::lock(&(*registry.state_for_test())).reserved, 0);
    assert!(worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::owner_cleanup::registry::lock(&(*registry.state_for_test())).entries.is_empty());
}
