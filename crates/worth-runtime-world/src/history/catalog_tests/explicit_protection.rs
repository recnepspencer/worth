use crate::history::reclamation::CompositeHistoryReclamationRequest;
use crate::history::CompositeRuntimeWorldCommit;

use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::fixtures::{history_contract, linear_history};

#[test]
fn explicit_commit_protection_is_exact_even_for_equal_basis_commits() {
    let (fixture, commits) = linear_history(2);
    let root = commits[0].clone();
    let successor = commits[1].clone();
    assert_eq!(root.basis(), successor.basis());
    let owner = root.identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(2, u64::MAX));
    catalog
        .append(root.clone(), fixture.history_pins(&(root.clone())))
        .expect("root install");
    catalog
        .append(
            successor.clone(),
            fixture.history_pins(&(successor.clone())),
        )
        .expect("successor install");

    let obligation = catalog
        .protect_explicit_commit(&successor)
        .expect("installed commit protection");
    assert_eq!(obligation.commit_identity(), successor.identity());
    assert_eq!(obligation.owner_identity(), owner);
    assert!(obligation.matches_commit(&successor));
    assert!(!obligation.matches_commit(&root));
}

#[test]
fn explicit_protection_denials_are_pre_effect() {
    let (_foreign_owner, foreign_commits) = linear_history(1);
    let (mut fixture, commits) = linear_history(1);
    let root = commits[0].clone();
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(1, u64::MAX),
    );
    catalog
        .append(root.clone(), fixture.history_pins(&(root.clone())))
        .expect("root install");
    let before = catalog.counters();

    assert!(matches!(
        catalog.protect_explicit_commit(&foreign_commits[0]),
        Err(CompositeHistoryCatalogDenial::ForeignOwner { .. })
    ));
    let unknown_identity = fixture
        .authority
        .issuer_mut()
        .composite_commit()
        .expect("same-owner unknown identity");
    let unknown = CompositeRuntimeWorldCommit::from_root_bootstrap(
        unknown_identity.clone(),
        root.basis().clone(),
        fixture
            .authority
            .issuer_mut()
            .bootstrap_attempt()
            .expect("unknown attempt"),
        None,
    )
    .expect("same-owner unknown commit");
    assert!(matches!(
        catalog.protect_explicit_commit(&unknown),
        Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(target))
            if target == unknown_identity
    ));

    let after = catalog.counters();
    assert_eq!(
        after.direct_protection_acquisitions(),
        before.direct_protection_acquisitions()
    );
    assert_eq!(
        after.direct_protection_releases(),
        before.direct_protection_releases()
    );
}

#[test]
fn product_head_and_explicit_consumer_release_independently() {
    let (fixture, commits) = linear_history(2);
    let root = commits[0].clone();
    let leaf = commits[1].clone();
    let owner = root.identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(2, u64::MAX));
    catalog
        .append((root).clone(), fixture.history_pins(&(root)))
        .expect("root install");
    catalog
        .append(leaf.clone(), fixture.history_pins(&(leaf.clone())))
        .expect("leaf install");
    let before = catalog.counters();

    let product_head = catalog
        .protect_product_head(&leaf)
        .expect("product-head protection");
    let explicit = catalog
        .protect_explicit_commit(&leaf)
        .expect("explicit protection");
    assert_eq!(
        catalog.counters().direct_protection_acquisitions(),
        before.direct_protection_acquisitions() + 2
    );

    drop(product_head);
    let moved = explicit;
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before.direct_protection_releases() + 1
    );
    let blocked = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner,
            vec![leaf.identity().clone()],
            1,
        ))
        .expect("explicit consumer remains live");
    assert_eq!(blocked.skipped_protected(), 1);

    drop(moved);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before.direct_protection_releases() + 2
    );
    let reclaimed = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner,
            vec![leaf.identity().clone()],
            1,
        ))
        .expect("final release permits reclamation");
    assert_eq!(reclaimed.reclaimed_commits(), &[leaf.identity().clone()]);
}
