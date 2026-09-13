use std::sync::Arc;

use crate::history::reclamation::CompositeHistoryReclamationRequest;
use crate::history::CompositeRuntimeWorldCommit;

use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::fixtures::{history_contract, linear_history};

#[test]
fn reserved_parent_dependency_survives_install_without_reacquisition() {
    let (owner, commits) = linear_history(2);
    let root = commits[0].clone();
    let child = commits[1].clone();
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(2, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("root install");

    let slot = catalog.reserve(child.as_ref()).expect("child reservation");
    let reserved_counters = catalog.counters();
    let blocked = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![root.identity().clone()],
            1,
        ))
        .expect("reserved child protects parent");
    assert_eq!(blocked.skipped_with_descendant_dependencies(), 1);

    let installed = slot
        .install(child.clone(), owner.history_pins(&child))
        .expect("child install");
    let installed_counters = catalog.counters();
    assert_eq!(installed.identity(), child.identity());
    assert_eq!(
        installed_counters.dependency_increments(),
        reserved_counters.dependency_increments()
    );
    assert_eq!(
        installed_counters.dependency_decrements(),
        reserved_counters.dependency_decrements()
    );

    let still_blocked = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![root.identity().clone()],
            1,
        ))
        .expect("installed child preserves parent edge");
    assert_eq!(still_blocked.skipped_with_descendant_dependencies(), 1);
}

#[test]
fn reservation_drop_and_mismatched_install_release_exactly_once() {
    let (owner, commits) = linear_history(3);
    let root = commits[0].clone();
    let child = commits[1].clone();
    let wrong = commits[2].clone();
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(3, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("root install");

    let before_drop = catalog.counters();
    drop(catalog.reserve(child.as_ref()).expect("child reservation"));
    let after_drop = catalog.counters();
    assert_eq!(
        after_drop.dependency_increments() - before_drop.dependency_increments(),
        1
    );
    assert_eq!(
        after_drop.dependency_decrements() - before_drop.dependency_decrements(),
        1
    );
    assert_eq!(catalog.reserved_len(), 0);
    assert_eq!(catalog.metadata_ledger().reservation_resident(), 0);
    assert_eq!(catalog.metadata_ledger().promised_installation(), 0);

    let before_mismatched_reserve = catalog.counters();
    let slot = catalog
        .reserve(child.as_ref())
        .expect("retry child reservation");
    let after_mismatched_reserve = catalog.counters();
    assert_eq!(
        after_mismatched_reserve.dependency_increments()
            - before_mismatched_reserve.dependency_increments(),
        1
    );
    assert!(matches!(
        slot.install((wrong).clone(), owner.history_pins(&(wrong))),
        Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch)
    ));
    let after_mismatched_install = catalog.counters();
    assert_eq!(
        after_mismatched_install.dependency_increments()
            - after_mismatched_reserve.dependency_increments(),
        0
    );
    assert_eq!(
        after_mismatched_install.dependency_decrements()
            - after_mismatched_reserve.dependency_decrements(),
        1
    );
    assert_eq!(
        after_mismatched_install.metadata_releases() - after_mismatched_reserve.metadata_releases(),
        1
    );
    assert_eq!(catalog.reserved_len(), 0);
    assert_eq!(catalog.len(), 1);
    let reclaimed = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![root.identity().clone()],
            1,
        ))
        .expect("failed install released the parent edge");
    assert_eq!(reclaimed.reclaimed_commits(), &[root.identity().clone()]);
}

#[test]
fn reservation_denials_keep_foreign_duplicate_root_and_parent_ordered() {
    let (_foreign_owner, foreign_commits) = linear_history(1);
    let (mut owner, commits) = linear_history(2);
    let first_root = commits[0].clone();
    let child = commits[1].clone();
    let second_root = Arc::new(
        CompositeRuntimeWorldCommit::from_root_bootstrap(
            owner
                .authority
                .issuer_mut()
                .composite_commit()
                .expect("second root identity"),
            first_root.basis().clone(),
            owner
                .authority
                .issuer_mut()
                .bootstrap_attempt()
                .expect("second root attempt"),
            None,
        )
        .expect("second root"),
    );
    let catalog = CompositeHistoryCatalog::new(
        first_root.identity().owner_identity(),
        history_contract(3, u64::MAX),
    );

    assert!(matches!(
        catalog.reserve(foreign_commits[0].as_ref()),
        Err(CompositeHistoryCatalogDenial::ForeignOwner { .. })
    ));
    let first_root_slot = catalog.reserve(first_root.as_ref()).expect("root slot");
    assert!(matches!(
        catalog.reserve(first_root.as_ref()),
        Err(CompositeHistoryCatalogDenial::DuplicateCommit)
    ));
    assert!(matches!(
        catalog.reserve(second_root.as_ref()),
        Err(CompositeHistoryCatalogDenial::RootAlreadyInstalled)
    ));
    drop(first_root_slot);

    assert!(matches!(
        catalog.reserve(child.as_ref()),
        Err(CompositeHistoryCatalogDenial::MissingParent(_))
    ));
}
