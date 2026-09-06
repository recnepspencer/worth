use crate::history::reclamation::{CompositeHistoryReclamationRequest, HistoryReclamationDenial};
use crate::history::retention::HistoryProtectionClass;

use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::fixtures::{history_contract, linear_history};

#[test]
fn descendant_dependencies_and_direct_protections_block_exact_targets() {
    let (owner, commits) = linear_history(3);
    let root = commits[0].clone();
    let ordinary = commits[1].clone();
    let leaf = commits[2].clone();
    let owner_identity = root.identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner_identity, history_contract(3, u64::MAX));
    for commit in [&root, &ordinary, &leaf] {
        catalog
            .append(commit.clone(), owner.history_pins(commit))
            .expect("chain install");
    }

    let blocked_root = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![root.identity().clone()],
            1,
        ))
        .expect("installed child blocks parent");
    assert_eq!(blocked_root.skipped_with_descendant_dependencies(), 1);

    let before_protection = catalog.counters();
    let protection = catalog
        .protect_exact(
            leaf.identity().clone(),
            HistoryProtectionClass::ExplicitObligation,
        )
        .expect("exact protection");
    let after_protection = catalog.counters();
    assert_eq!(
        after_protection.direct_protection_acquisitions()
            - before_protection.direct_protection_acquisitions(),
        1
    );
    let blocked_leaf = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![leaf.identity().clone()],
            1,
        ))
        .expect("direct protection blocks leaf");
    assert_eq!(blocked_leaf.skipped_protected(), 1);
    drop(protection);
    let after_protection_drop = catalog.counters();
    assert_eq!(
        after_protection_drop.direct_protection_releases()
            - after_protection.direct_protection_releases(),
        1
    );

    let reclaimed_leaf = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![leaf.identity().clone()],
            1,
        ))
        .expect("released protection permits leaf reclaim");
    assert_eq!(
        reclaimed_leaf.reclaimed_commits(),
        &[leaf.identity().clone()]
    );
    catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![ordinary.identity().clone()],
            1,
        ))
        .expect("reclaimed leaf releases ordinary dependency");
    let reclaimed_root = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![root.identity().clone()],
            1,
        ))
        .expect("reclaimed ordinary releases root dependency");
    assert_eq!(
        reclaimed_root.reclaimed_commits(),
        &[root.identity().clone()]
    );
}

#[test]
fn malformed_bounded_prefix_is_rejected_before_reachability_or_mutation() {
    let (mut owner, commits) = linear_history(2);
    let root = commits[0].clone();
    let child = commits[1].clone();
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(2, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("root install");
    catalog
        .append(child.clone(), owner.history_pins(&child))
        .expect("child install");
    let unknown = owner
        .authority
        .issuer_mut()
        .composite_commit()
        .expect("unknown identity");
    let before = catalog.counters();
    let denial = catalog.reclaim_batch(CompositeHistoryReclamationRequest::new(
        root.identity().owner_identity(),
        vec![unknown.clone(), child.identity().clone()],
        1,
    ));
    assert_eq!(
        denial,
        Err(HistoryReclamationDenial::UnknownCandidate(unknown.clone()))
    );
    assert_eq!(catalog.len(), 2);
    let after = catalog.counters();
    assert_eq!(after.reachability_lookups(), before.reachability_lookups());
    assert_eq!(after.metadata_releases(), before.metadata_releases());
    assert_eq!(
        after.dependency_decrements(),
        before.dependency_decrements()
    );
    assert!(catalog.lookup(child.identity()).is_some());

    let duplicate = catalog.reclaim_batch(CompositeHistoryReclamationRequest::new(
        root.identity().owner_identity(),
        vec![child.identity().clone(), child.identity().clone()],
        2,
    ));
    assert_eq!(
        duplicate,
        Err(HistoryReclamationDenial::DuplicateCandidate(
            child.identity().clone()
        ))
    );
    assert_eq!(catalog.len(), 2);

    let bounded = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![child.identity().clone(), unknown],
            1,
        ))
        .expect("candidate suffix beyond the bound is not inspected");
    assert_eq!(bounded.reclaimed_commits(), &[child.identity().clone()]);
    assert_eq!(catalog.len(), 1);
}

#[test]
fn zero_batch_does_no_candidate_index_allocation_or_mutation_work() {
    let (mut owner, commits) = linear_history(2);
    let root = commits[0].clone();
    let child = commits[1].clone();
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(2, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("root install");
    catalog
        .append(child.clone(), owner.history_pins(&child))
        .expect("child install");
    let unknown = owner
        .authority
        .issuer_mut()
        .composite_commit()
        .expect("unknown identity");
    let before = catalog.counters();
    let ledger_before = catalog.metadata_ledger();
    let outcome = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![unknown],
            0,
        ))
        .expect("zero batch is a valid no-op");
    let after = catalog.counters();
    assert_eq!(outcome.examined(), 0);
    assert_eq!(outcome.reclaimed_commits(), &[]);
    assert_eq!(
        after.candidate_validations(),
        before.candidate_validations()
    );
    assert_eq!(after.reachability_lookups(), before.reachability_lookups());
    assert_eq!(after.metadata_releases(), before.metadata_releases());
    assert_eq!(
        after.dependency_decrements(),
        before.dependency_decrements()
    );
    assert_eq!(catalog.metadata_ledger(), ledger_before);
    assert_eq!(catalog.len(), 2);
}

#[test]
fn exact_protection_validation_rejects_foreign_and_unknown_occurrences() {
    let (_foreign_owner, foreign_commits) = linear_history(1);
    let (mut owner, commits) = linear_history(1);
    let root = commits[0].clone();
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(1, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("root install");
    let before_denials = catalog.counters();
    assert!(matches!(
        catalog.protect_product_head(&foreign_commits[0]),
        Err(CompositeHistoryCatalogDenial::ForeignOwner { .. })
    ));
    let unknown = owner
        .authority
        .issuer_mut()
        .composite_commit()
        .expect("unknown identity");
    assert!(matches!(
        catalog.protect_product_head(&crate::history::CompositeRuntimeWorldCommit::from_root_bootstrap(
            unknown.clone(),
            root.basis().clone(),
            owner.authority.issuer_mut().bootstrap_attempt().expect("unknown attempt"),
            None,
        ).expect("same-owner unknown commit")),
        Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(target)) if target == unknown
    ));
    let after_denials = catalog.counters();
    assert_eq!(
        after_denials.direct_protection_acquisitions(),
        before_denials.direct_protection_acquisitions()
    );
    assert_eq!(
        after_denials.direct_protection_releases(),
        before_denials.direct_protection_releases()
    );
}

#[test]
fn product_head_protection_proves_one_exact_installed_occurrence() {
    let (owner, commits) = linear_history(2);
    let root = commits[0].clone();
    let successor = commits[1].clone();
    let owner_identity = root.identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner_identity, history_contract(2, u64::MAX));
    catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("root install");
    catalog
        .append(successor.clone(), owner.history_pins(&successor))
        .expect("successor install");

    let before = catalog.counters();
    let protection = catalog
        .protect_product_head(&successor)
        .expect("installed product head is protectable");
    assert_eq!(protection.commit_identity(), successor.identity());
    assert_eq!(protection.owner_identity(), owner_identity);
    assert!(protection.matches_commit(&successor));
    assert!(!protection.matches_commit(&root));
    assert_eq!(
        catalog.counters().direct_protection_acquisitions(),
        before.direct_protection_acquisitions() + 1
    );

    let moved = protection;
    let blocked = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![successor.identity().clone()],
            1,
        ))
        .expect("live product head blocks reclamation");
    assert_eq!(blocked.skipped_protected(), 1);
    drop(moved);
    assert_eq!(
        catalog.counters().direct_protection_releases(),
        before.direct_protection_releases() + 1
    );
    let reclaimed = catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![successor.identity().clone()],
            1,
        ))
        .expect("final drop permits reclamation");
    assert_eq!(
        reclaimed.reclaimed_commits(),
        &[successor.identity().clone()]
    );
}
