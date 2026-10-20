//! Ordinary retention has no application quota, while every native claim stays owned.
use super::super::fork_creation::{current_root_observation, seed_relational_source};
use super::super::*;

#[test]
fn no_policy_world_keeps_distinct_originals_history_and_final_drop_custody() {
    let mut fixture = crate::branch::reference_test_fixture::real_fixture(8, 8);
    let budgets = RuntimeWorldBudgets::without_policy_limits();
    for policy in [
        budgets.live_product_branches(),
        budgets.retained_composite_commits(),
        budgets.history_metadata_bytes(),
        budgets.active_observations(),
        budgets.active_publication_attempts(),
        budgets.retained_product_unpublished_records(),
        budgets.retained_partial_metadata_bytes(),
        budgets.unique_exact_component_pins(),
        budgets.in_flight_pin_acquisition_reservations(),
        budgets.owner_created_component_custody_records(),
    ] {
        assert_eq!(policy.get(), None);
    }
    let owner =
        TestOwner::new(fixture.owner_inputs(budgets, RuntimeWorldClock::from_source(FixedClock)))
            .unwrap();
    let mut current = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed.product_branch().clone(),
        other => panic!("real bootstrap: {other:?}"),
    };
    let mut originals = Vec::new();
    // Workload size, not an installed ceiling: all original observations remain held.
    for _ in 0..600 {
        originals.push(
            owner
                .observe_product_branch(current.branch_identity())
                .unwrap(),
        );
    }
    assert_eq!(owner.state.retention.active_observation_count(), 601);
    let clone = originals.last().unwrap().clone();
    drop(originals.pop());
    assert_eq!(owner.state.retention.active_observation_count(), 601);
    drop(clone);
    assert_eq!(owner.state.retention.active_observation_count(), 600);

    let attempts: Vec<_> = (0..20)
        .map(|_| owner.state.publication_capacity.reserve().unwrap())
        .collect();
    let recoveries: Vec<_> = (0..20)
        .map(|_| {
            owner
                .state
                .recovery
                .reserve_product_unpublished(owner.owner_identity())
                .unwrap()
        })
        .collect();
    assert_eq!(owner.state.publication_capacity.active(), attempts.len());
    drop((attempts, recoveries));
    assert_eq!(owner.state.publication_capacity.active(), 0);

    for index in 0..600 {
        seed_relational_source(&owner, &mut fixture, current.clone());
        let next = current_root_observation(&owner);
        assert_ne!(current.selected_commit(), next.selected_commit());
        originals.push(current);
        current = next;
        let branch = create_reused_branch(&owner, &current, reuse_intent(&format!("held-{index}")));
        originals.push(branch);
    }
    assert_eq!(owner.state.history.len(), 601);
    assert_eq!(owner.state.branches.branch_count(), 601);
    assert!(owner.state.retention.unique_pin_count() > 512);
    let count = owner.state.retention.active_observation_count();
    assert_eq!(count, originals.len() + 1);
    let report = owner.close().unwrap();
    assert_eq!(report.outstanding_observations(), count);
    // Close releases the owner heads, not these original observations.
    assert_eq!(owner.state.retention.active_observation_count(), count);
    assert!(owner.state.retention.active_component_obligation_count() > 0);
    let retention = owner.state.retention.clone();
    let before_drop = retention.active_component_obligation_count();
    drop((originals, current));
    assert_eq!(retention.active_observation_count(), 0);
    assert_eq!(
        retention.active_component_obligation_count(),
        before_drop - 2 * count
    );
    // The closed owner still owns its root/history. Their backing releases when
    // that owner drops, rather than by discarding any caller's observation.
    drop(owner);
    assert_eq!(retention.active_component_obligation_count(), 0);
    println!("no-policy: 601 initial originals; 601 retained commits/branches; {count} originals survive close; final drop releases all claims");
}
