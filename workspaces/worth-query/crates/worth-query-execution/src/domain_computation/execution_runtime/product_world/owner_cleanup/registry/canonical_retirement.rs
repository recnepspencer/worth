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
    let registry = WorthQueryProductBranchOwnerCleanupRegistry::new(Some(3));
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

#[test]
fn absent_cleanup_policy_keeps_reservations_until_drop() {
    let registry = super::WorthQueryProductBranchOwnerCleanupRegistry::new(None);
    let held: Vec<_> = (0..600)
        .map(|_| registry.reserve(super::CleanupScope::Application).unwrap())
        .collect();
    assert_eq!(super::lock(&registry.state).reserved, held.len());
    drop(held);
    assert_eq!(super::lock(&registry.state).reserved, 0);
    assert!(super::lock(&registry.state).entries.is_empty());
}

#[test]
fn funded_hidden_cleanup_rolls_back_after_stop_without_room_or_visibility() {
    use worth_execution::{
        CancellationSource, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
    };
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let selected = runtime
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let branch = selected.branch_identity();
    let occurrence = selected.observation().lifecycle_incarnation();
    let registry = WorthQueryProductBranchOwnerCleanupRegistry::new(Some(1));
    let cancellation = CancellationSource::new();
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(2_000_000),
                Some(1_000_000),
            ),
        ),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap();
    let reservation = registry
        .reserve_funded_retirement(branch, occurrence, ExecutionRequest::leased(&lease))
        .unwrap();
    assert_eq!(super::lock(&registry.state).reserved, 1);
    assert_eq!(super::lock(&registry.state).entries.len(), 1);
    assert!(registry.pending_identities().is_empty());
    assert!(registry
        .pending_application_retired_product_occurrences()
        .is_empty());
    assert!(lease.charged_memory_bytes() > 0);
    let blocker = lease
        .reserve_memory(lease.available_memory_bytes().unwrap())
        .unwrap();
    cancellation.cancel();
    assert!(ExecutionRequest::leased(&lease).consult().is_err());
    drop(reservation);
    assert_eq!(super::lock(&registry.state).reserved, 0);
    assert!(super::lock(&registry.state).entries.is_empty());
    assert!(registry.pending_identities().is_empty());
    assert_eq!(lease.charged_memory_bytes(), blocker.bytes());
    drop(blocker);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
