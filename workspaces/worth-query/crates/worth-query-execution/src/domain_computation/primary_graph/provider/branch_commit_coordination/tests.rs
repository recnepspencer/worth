use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use super::{
    WorthQueryApplicationBranchCommitCoordinator as Coordinator,
    WorthQueryBranchCommitLaneDenial as Denial,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::tests::fixture::installed_capability_authorization_world;

#[test]
fn weak_entry_and_live_lane_share_one_refundable_retained_claim() {
    let world = installed_capability_authorization_world();
    let selected = world.selected_product();
    let observation = selected.product().observation();
    let occurrence = observation.lifecycle_incarnation();

    let measured = Coordinator::new(usize::MAX);
    let lane = measured.lane_for(observation).unwrap();
    let exact = measured.retained_bytes();
    assert!(exact > 0);
    drop(lane);
    assert_eq!(measured.retained_bytes(), exact);
    measured.retire(occurrence);
    assert_eq!(measured.retained_bytes(), 0);

    let denied = Coordinator::new(exact - 1);
    assert!(matches!(
        denied.lane_for(observation),
        Err(Denial::RetainedCapacityExhausted { .. })
    ));
    assert_eq!(denied.entry_count(), 0);
    assert_eq!(denied.retained_bytes(), 0);

    let exact_owner = Coordinator::new(exact);
    let lane = exact_owner.lane_for(observation).unwrap();
    exact_owner.retire(occurrence);
    assert_eq!(exact_owner.entry_count(), 0);
    assert_eq!(exact_owner.retained_bytes(), exact);
    drop(lane);
    assert_eq!(exact_owner.retained_bytes(), 0);
}

#[test]
fn admitted_lane_denies_before_creating_a_weak_entry() {
    let world = installed_capability_authorization_world();
    let selected = world.selected_product();
    let observation = selected.product().observation();
    let owner = Coordinator::new(usize::MAX);
    let mut short = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 100,
        maximum_preparation_bytes: 0,
    });
    assert!(matches!(
        owner.lane_for_admitted(observation, &mut short),
        Err(Denial::Preparation(
            CompanionPreflightStop::PreparationMemoryExhausted { .. }
        ))
    ));
    assert_eq!(owner.entry_count(), 0);
    assert_eq!(owner.retained_bytes(), 0);

    let mut funded = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 100,
        maximum_preparation_bytes: 16 * 1024,
    });
    let lane = owner.lane_for_admitted(observation, &mut funded).unwrap();
    assert!(funded.charged_work() > 0);
    assert!(funded.charged_bytes() > 0);
    drop(lane);
    owner.retire(observation.lifecycle_incarnation());
    assert_eq!(owner.retained_bytes(), 0);
}

#[test]
fn expired_weak_replacement_keeps_the_entry_claim_until_retirement() {
    let world = installed_capability_authorization_world();
    let selected = world.selected_product();
    let observation = selected.product().observation();
    let owner = Coordinator::new(usize::MAX);
    let first = owner.lane_for(observation).unwrap();
    let retained = owner.retained_bytes();
    drop(first);
    assert_eq!(owner.retained_bytes(), retained);

    let replacement = owner.lane_for(observation).unwrap();
    assert_eq!(owner.entry_count(), 1);
    assert_eq!(owner.retained_bytes(), retained);
    owner.retire(observation.lifecycle_incarnation());
    assert_eq!(owner.retained_bytes(), retained);
    drop(replacement);
    assert_eq!(owner.retained_bytes(), 0);
}
