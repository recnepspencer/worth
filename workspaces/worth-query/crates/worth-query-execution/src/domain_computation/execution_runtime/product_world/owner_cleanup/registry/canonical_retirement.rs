//! Cleanup issuance never decides the release order of genuine occurrences.
use super::*;
use worth_runtime_world::facade::*;

#[test]
fn retirement_releases_in_branch_then_incarnation_order() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let root = runtime
        .admit_product_branch(runtime.default_branch())
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
            .create_product_branch(
                &root,
                None,
                intent,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("a genuine child occurrence is created");
        };
        let occurrence = WorthQueryRetiredProductOccurrence::new(
            child.branch_identity().clone(),
            child.lifecycle_incarnation(),
        );
        let observed = runtime
            .admit_product_occurrence(child.lifecycle_incarnation())
            .unwrap();
        let report = runtime.retire_product_branch(&observed).unwrap();
        (occurrence, report)
    };
    let (zulu, zulu_report) = child("zulu");
    let (alpha_first, first_report) = child("alpha");
    let (alpha_second, second_report) = child("alpha");
    assert_eq!(alpha_first.branch(), alpha_second.branch());
    assert!(alpha_first.incarnation() < alpha_second.incarnation());
    let registry = WorthQueryProductBranchOwnerCleanupRegistry::new(3);
    for (occurrence, report) in [
        (zulu.clone(), zulu_report),
        (alpha_second.clone(), second_report),
        (alpha_first.clone(), first_report),
    ] {
        registry
            .reserve(CleanupScope::ApplicationRetirement(occurrence))
            .unwrap()
            .install_retirement(report);
    }
    let actual = registry
        .pending_application_retired_product_occurrences()
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
