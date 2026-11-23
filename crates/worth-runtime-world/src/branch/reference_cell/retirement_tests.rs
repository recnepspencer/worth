//! Deterministic observation-fencing tests for in-flight and recovery-required retirement (B2).
use std::sync::Arc;

use super::super::{
    ProductBranchReferenceCell, ProductBranchReferenceObservationFailure,
};
use crate::branch::reference_test_fixture as fixture;

#[test]
fn retiring_cell_rejects_fresh_live_observation_during_in_flight_and_recovery_required() {
    let (mut fixture, catalog, root) = fixture::installed_root();
    let snapshot = fixture::initial_snapshot(&mut fixture, Arc::clone(&root));
    let protection = fixture::product_head_protection(&fixture, &catalog, snapshot);
    let cell = ProductBranchReferenceCell::new(protection).expect("protected cell admission");

    // Pre-existing observation remains valid across retirement phases.
    let prior = cell
        .observe(&catalog, &fixture.owner)
        .expect("live observation before retirement");

    // Enter RetirementInFlight (pending Branches append).
    let (permit, retiring_snapshot) = cell
        .begin_durable_retirement()
        .expect("enter RetirementInFlight");
    assert_eq!(retiring_snapshot.selected_commit(), prior.selected_commit());
    assert!(cell.is_retired_or_retiring());

    let releases_before = catalog.counters().direct_protection_releases();
    assert!(matches!(
        cell.observe(&catalog, &fixture.owner),
        Err(ProductBranchReferenceObservationFailure::Retired)
    ));
    // Prior observation and explicit history reads remain intact.
    assert_eq!(
        catalog.lookup(prior.selected_commit()).unwrap().identity(),
        prior.selected_commit()
    );
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        releases_before
    );

    // Proven no-effect clears the barrier and restores live observation.
    permit.proven_no_effect();
    assert!(!cell.is_retired_or_retiring());
    let resumed = cell
        .observe(&catalog, &fixture.owner)
        .expect("live observation resumes after proven_no_effect");
    drop(resumed);

    // Dropping an armed permit (indeterminate append) enters RetirementRecoveryRequired.
    let (indeterminate_permit, _) = cell
        .begin_durable_retirement()
        .expect("re-enter RetirementInFlight");
    drop(indeterminate_permit);
    assert!(cell.is_retired_or_retiring());
    assert!(matches!(
        cell.observe(&catalog, &fixture.owner),
        Err(ProductBranchReferenceObservationFailure::Retired)
    ));
    // Prior observation and explicit retained history still remain readable.
    assert_eq!(
        catalog.lookup(prior.selected_commit()).unwrap().identity(),
        prior.selected_commit()
    );
    drop(prior);
}
