use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::fixtures::{history_contract, linear_history};
use crate::history::reclamation::CompositeHistoryReclamationRequest;

#[test]
fn protected_installation_denial_keeps_the_original_reservation_for_retry() {
    let (owner, commits) = linear_history(3);
    let root = &commits[0];
    let child = &commits[1];
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(3, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(root))
        .unwrap();
    let mut reserved = catalog.reserve(child).unwrap();
    let before = catalog.counters();
    assert!(matches!(
        reserved.try_install_product_head(
            (commits[2].clone()).clone(),
            owner.history_pins(&(commits[2].clone()))
        ),
        Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch)
    ));
    assert_eq!(catalog.reserved_len(), 1);
    assert_eq!(catalog.len(), 1);
    assert_eq!(
        catalog.counters().metadata_releases(),
        before.metadata_releases()
    );
    let protection = reserved
        .try_install_product_head(child.clone(), owner.history_pins(child))
        .unwrap();
    assert_eq!(protection.commit_identity(), child.identity());
    assert_eq!(catalog.reserved_len(), 0);
    assert_eq!(catalog.len(), 2);
    assert_eq!(
        catalog.counters().dependency_increments(),
        before.dependency_increments()
    );
    assert_eq!(
        catalog.counters().direct_protection_acquisitions(),
        before.direct_protection_acquisitions() + 1
    );
    let retained_claims = owner.retention.snapshot(0).component_obligations();
    let report = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![child.identity().clone()],
            1,
        ))
        .unwrap();
    assert_eq!(report.skipped_protected(), 1);
    drop(protection);
    let report = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![child.identity().clone()],
            1,
        ))
        .unwrap();
    assert_eq!(report.reclaimed_commits(), &[child.identity().clone()]);
    assert_eq!(
        owner.retention.snapshot(0).component_obligations(),
        retained_claims - 2,
        "completed reservation must not keep reclaimed history pins alive"
    );
    drop(reserved);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before.direct_protection_releases() + 1
    );
}

#[test]
fn protected_installation_cannot_promote_the_same_reservation_twice() {
    let (owner, commits) = linear_history(1);
    let root = &commits[0];
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(1, u64::MAX),
    );
    let mut reserved = catalog.reserve(root).unwrap();
    let protection = reserved
        .try_install_product_head(root.clone(), owner.history_pins(root))
        .unwrap();
    let before = catalog.counters();
    assert!(matches!(
        reserved.try_install_product_head(root.clone(), owner.history_pins(root)),
        Err(CompositeHistoryCatalogDenial::ReservationMissing)
    ));
    drop(reserved);
    assert_eq!(catalog.len(), 1);
    assert_eq!(
        catalog.counters().metadata_promotions(),
        before.metadata_promotions()
    );
    assert_eq!(
        catalog.counters().direct_protection_acquisitions(),
        before.direct_protection_acquisitions()
    );
    drop(protection);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before.direct_protection_releases() + 1
    );
}
