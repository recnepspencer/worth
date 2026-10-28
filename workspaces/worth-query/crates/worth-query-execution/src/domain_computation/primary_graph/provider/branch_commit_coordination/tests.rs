use super::WorthQueryApplicationBranchCommitCoordinator as Coordinator;

use crate::domain_computation::primary_graph::tests::fixture::installed_capability_authorization_world;

#[test]
fn same_occurrence_shares_a_lane_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .unwrap();
    let observation = selected.product().observation();
    let owner = Coordinator::new();
    let first = owner.lane_for(observation);
    let second = owner.lane_for(observation);
    assert!(std::sync::Arc::ptr_eq(&first, &second));
    assert_eq!(owner.entry_count(), 1);
    owner.retire(observation.lifecycle_incarnation());
    assert_eq!(owner.entry_count(), 0);
    drop((first, second));
}

#[test]
fn expired_weak_replacement_keeps_the_entry_claim_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let observation = selected.product().observation();
    let owner = Coordinator::new();
    let first = owner.lane_for(observation);

    drop(first);

    let replacement = owner.lane_for(observation);
    assert_eq!(owner.entry_count(), 1);

    owner.retire(observation.lifecycle_incarnation());

    drop(replacement);
}

#[test]
fn funded_lane_keeps_original_backing_through_strong_and_dead_weak_owners() {
    use worth_execution::{
        CancellationSource, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
    };
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let world = installed_capability_authorization_world();
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .unwrap();
    let occurrence = selected.product().observation().lifecycle_incarnation();
    let owner = Coordinator::new();
    let cancellation = CancellationSource::new();
    let policy = |bytes| {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(bytes), Some(1_000_000)),
        )
    };
    let zero = ExecutionResourceLease::serial(LeaseRequest {
        policy: policy(0),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap();
    assert!(matches!(
        owner.lane_for_request(occurrence, ExecutionRequest::leased(&zero)),
        Err(super::BranchCoordinationDenial::Request(_))
    ));
    assert_eq!(owner.entry_count(), 0);

    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: policy(2_000_000),
        cancellation: cancellation.token(),
        deadline: None,
    })
    .unwrap();
    let request = ExecutionRequest::leased(&lease);
    let first = owner.lane_for_request(occurrence, request).unwrap();
    let same = owner.lane_for_request(occurrence, request).unwrap();
    assert!(std::sync::Arc::ptr_eq(&first, &same));
    drop((first, same));
    assert!(
        lease.charged_memory_bytes() > 0,
        "dead Weak still owns actual Arc backing"
    );
    owner.retire(occurrence);
    assert_eq!(lease.charged_memory_bytes(), 0);

    let still_live = owner.lane_for_request(occurrence, request).unwrap();
    owner.retire(occurrence);
    assert!(
        lease.charged_memory_bytes() > 0,
        "live lane still owns actual Arc backing after roster removal"
    );
    drop(still_live);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
