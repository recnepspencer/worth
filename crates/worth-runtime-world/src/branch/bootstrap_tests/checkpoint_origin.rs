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
    assert_eq!(retired.retention_keys(), &keys);
    assert_eq!(fixture.owner.active_component_obligation_count(), 0);
    for key in &keys {
        let entry = fixture.owner.inspect_key(key).unwrap().unwrap();
        assert!(!entry.owner_lease_present);
        assert!(entry.dependencies.is_zero());
    }
    let reclaimed = fixture
        .owner
        .reclaim_keys(retired.retention_keys(), 2)
        .unwrap();
    assert_eq!(reclaimed.reclaimed, 2);
}
