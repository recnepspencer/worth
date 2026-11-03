//! A durable origin retains the actual shared component leases until consumed.

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
