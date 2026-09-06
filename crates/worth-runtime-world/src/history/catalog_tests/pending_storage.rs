use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::fixtures::{history_contract, linear_history};
use crate::history::reclamation::{CompositeHistoryReclamationRequest, HistoryReclamationDenial};

#[test]
fn preallocated_slots_are_invisible_until_promotion_and_release_once_on_drop() {
    let (owner, commits) = linear_history(3);
    let root = &commits[0];
    let child = &commits[1];
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(2, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(root))
        .unwrap();
    let before_metadata = catalog.metadata_ledger();
    let before_counters = catalog.counters();
    let reserved = catalog.reserve(child).unwrap();
    assert_eq!(catalog.len(), 1);
    assert_eq!(catalog.reserved_len(), 1);
    assert!(catalog.lookup(child.identity()).is_none());
    assert!(matches!(
        catalog.protect_product_head(child),
        Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(_))
    ));
    assert!(matches!(
        catalog.protect_explicit_commit(child),
        Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(_))
    ));
    assert!(matches!(
        catalog.reserve(&commits[2]),
        Err(CompositeHistoryCatalogDenial::MissingParent(_))
    ));
    assert!(matches!(
        catalog.reclaim_batch(CompositeHistoryReclamationRequest::new(
            root.identity().owner_identity(),
            vec![child.identity().clone()],
            1,
        )),
        Err(HistoryReclamationDenial::UnknownCandidate(_))
    ));
    // This inspects physical slot residence independently from the public
    // installed count: the lookup index already contains the pending key.
    {
        let state = super::super::support::lock_state(&catalog.state);
        assert!(state
            .entries
            .get(child.identity())
            .is_some_and(|slot| slot.get().is_none()));
        assert_eq!(state.entries.len(), 2);
    }
    assert_eq!(
        catalog.counters().reachability_rows_installed(),
        before_counters.reachability_rows_installed()
    );
    drop(reserved);
    assert_eq!(catalog.metadata_ledger(), before_metadata);
    assert_eq!(catalog.reserved_len(), 0);
    assert_eq!(catalog.len(), 1);
    assert_eq!(
        catalog.counters().dependency_decrements(),
        before_counters.dependency_decrements() + 1
    );
    assert!(!super::super::support::lock_state(&catalog.state)
        .entries
        .contains_key(child.identity()));
    let reservation = catalog
        .reserve(child)
        .expect("dropping pending storage releases the exact count and key");
    reservation
        .install(child.clone(), owner.history_pins(child))
        .unwrap();
    assert_eq!(
        catalog.len(),
        2,
        "a reservation spends one catalog slot, not two"
    );
    assert!(catalog.lookup(child.identity()).is_some());
}

#[test]
fn carried_reservation_survives_index_growth_and_installs_the_exact_occurrence() {
    use crate::history::CompositeRuntimeWorldCommit;
    use crate::publication::CompositeOwnerExecutionResults;
    use std::sync::Arc;
    let (mut owner, commits) = linear_history(96);
    let root = &commits[0];
    let catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(97, u64::MAX),
    );
    catalog
        .append(root.clone(), owner.history_pins(root))
        .unwrap();
    let held = Arc::new(
        CompositeRuntimeWorldCommit::from_ordinary_publication(
            owner.authority.issuer_mut().composite_commit().unwrap(),
            root,
            root.basis().clone(),
            owner.authority.issuer_mut().publication_attempt().unwrap(),
            &CompositeOwnerExecutionResults::retained(),
            None,
        )
        .unwrap(),
    );
    let reserved = catalog.reserve(&held).unwrap();
    let (slot, capacity) = {
        let state = super::super::support::lock_state(&catalog.state);
        (
            Arc::clone(&state.entries[held.identity()]),
            state.entries.capacity(),
        )
    };
    for commit in &commits[1..] {
        catalog
            .append(commit.clone(), owner.history_pins(commit))
            .unwrap();
    }
    assert!(
        super::super::support::lock_state(&catalog.state)
            .entries
            .capacity()
            > capacity
    );
    assert!(slot.get().is_none());
    let writes = catalog.counters().reserved_entry_writes();
    reserved
        .install(held.clone(), owner.history_pins(&held))
        .unwrap();
    assert_eq!(catalog.counters().reserved_entry_writes(), writes + 1);
    assert_eq!(slot.get().unwrap().commit().identity(), held.identity());
    assert_eq!(slot.get().unwrap().commit().parent(), held.parent());
    assert_eq!(catalog.len(), 97);
    assert_eq!(catalog.reserved_len(), 0);
}
