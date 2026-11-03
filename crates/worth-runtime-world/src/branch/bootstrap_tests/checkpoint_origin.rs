//! A durable origin retains the actual shared component leases until consumed.
#[test]
fn checkpoint_origin_keeps_exact_component_leases_after_history_claims_retire() {
    let fixture = crate::branch::reference_test_fixture::real_fixture(2, 2);
    let head = fixture.owner.issue_product_head(&fixture.basis).unwrap();
    let history = head.fork_history(&fixture.basis).unwrap();
    let origin_pins = head.fork_history(&fixture.basis).unwrap();
    let keys = origin_pins.retention_keys();
    let acquired = fixture.owner.cost_snapshot().owner_acquisition_contacts();
    let origin = crate::durability::WorldCheckpointOrigin::new(&fixture.basis, origin_pins);
    assert_eq!(
        fixture.owner.cost_snapshot().owner_acquisition_contacts(),
        acquired
    );
    drop(head);
    drop(history);
    for key in &keys {
        let entry = fixture.owner.inspect_key(key).unwrap().unwrap();
        assert!(entry.owner_lease_present);
        assert_eq!(entry.dependencies.total(), 1);
    }
    assert_eq!(fixture.owner.active_component_obligation_count(), 2);
    let retired = origin.retire();
    let retired_keys: Vec<_> = retired.retention_keys().cloned().collect();
    assert_eq!(retired_keys, keys);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    for key in &keys {
        let entry = fixture.owner.inspect_key(key).unwrap().unwrap();
        assert!(!entry.owner_lease_present);
        assert!(entry.dependencies.is_zero());
    }
    let reclaimed = fixture.owner.reclaim_keys(&retired_keys, 2).unwrap();
    assert_eq!(reclaimed.reclaimed(), 2);
}

#[test]
fn original_observation_retirement_keeps_alias_veto_and_retries_busy_history_once() {
    use crate::branch::{reference_test_fixture as fixture, ProductBranchReferenceCell};
    let (mut fixture, catalog, root) = fixture::installed_root();
    let snapshot = fixture::initial_snapshot(&mut fixture, root);
    let protection = fixture::product_head_protection(&fixture, &catalog, snapshot);
    let cell = ProductBranchReferenceCell::new(protection).unwrap();
    let mut observation = cell.observe(&catalog, &fixture.owner).unwrap();
    let alias = observation.clone();
    let before = fixture.owner.active_component_obligation_count();
    assert!(observation.try_retire_for_close().is_err());
    assert_eq!(fixture.owner.active_component_obligation_count(), before);
    drop(alias);
    catalog.assert_observation_close_busy_preserves_history(&mut observation);
    assert_eq!(
        fixture.owner.active_component_obligation_count(),
        before - 2
    );
    drop(observation);
    assert_eq!(
        fixture.owner.active_component_obligation_count(),
        before - 2
    );
}
