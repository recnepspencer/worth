use crate::history::reclamation::CompositeHistoryReclamationRequest;

use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::allocation_oracle::AllocationOracle;
use super::fixtures::{history_contract, linear_history};

#[test]
fn allocation_ledger_matches_an_independent_oracle_across_lifecycles() {
    let (owner, commits) = linear_history(2);
    let root = commits[0].clone();
    let child = commits[1].clone();
    let owner_identity = root.identity().owner_identity();
    let maximum = AllocationOracle::installed_resident(root.as_ref())
        + AllocationOracle::reservation_resident(child.as_ref())
        + AllocationOracle::installed_resident(child.as_ref());
    let catalog =
        CompositeHistoryCatalog::new(owner_identity, history_contract(2, as_u64(maximum)));

    let root_slot = catalog.reserve(root.as_ref()).expect("root reservation");
    assert_ledger(
        catalog.metadata_ledger(),
        0,
        AllocationOracle::reservation_resident(root.as_ref()),
        AllocationOracle::installed_resident(root.as_ref()),
        AllocationOracle::reservation_plus_installation(root.as_ref()),
    );
    root_slot
        .install(root.clone(), owner.history_pins(&root))
        .expect("root installation");
    assert_ledger(
        catalog.metadata_ledger(),
        AllocationOracle::installed_resident(root.as_ref()),
        0,
        0,
        AllocationOracle::installed_resident(root.as_ref()),
    );

    let child_slot = catalog.reserve(child.as_ref()).expect("child reservation");
    assert_ledger(
        catalog.metadata_ledger(),
        AllocationOracle::installed_resident(root.as_ref()),
        AllocationOracle::reservation_resident(child.as_ref()),
        AllocationOracle::installed_resident(child.as_ref()),
        maximum,
    );
    child_slot
        .install(child.clone(), owner.history_pins(&child))
        .expect("child installation");
    assert_ledger(
        catalog.metadata_ledger(),
        AllocationOracle::installed_resident(root.as_ref())
            + AllocationOracle::installed_resident(child.as_ref()),
        0,
        0,
        AllocationOracle::installed_resident(root.as_ref())
            + AllocationOracle::installed_resident(child.as_ref()),
    );

    catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![child.identity().clone()],
            1,
        ))
        .expect("child reclaim");
    assert_ledger(
        catalog.metadata_ledger(),
        AllocationOracle::installed_resident(root.as_ref()),
        0,
        0,
        AllocationOracle::installed_resident(root.as_ref()),
    );
    catalog
        .reclaim_batch(CompositeHistoryReclamationRequest::new(
            owner_identity,
            vec![root.identity().clone()],
            1,
        ))
        .expect("root reclaim");
    assert_ledger(catalog.metadata_ledger(), 0, 0, 0, 0);
}

#[test]
fn exact_metadata_budget_succeeds_and_one_byte_less_denies_pre_effect() {
    let (owner, commits) = linear_history(1);
    let root = commits[0].clone();
    let exact = AllocationOracle::reservation_plus_installation(root.as_ref());
    assert!(exact > 0);

    let exact_catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(1, as_u64(exact)),
    );
    exact_catalog
        .append(root.clone(), owner.history_pins(&root))
        .expect("complete promised installation fits exact budget");
    assert_eq!(
        exact_catalog.metadata_ledger().installed_resident(),
        AllocationOracle::installed_resident(root.as_ref())
    );

    let less_catalog = CompositeHistoryCatalog::new(
        root.identity().owner_identity(),
        history_contract(1, as_u64(exact - 1)),
    );
    let denial = less_catalog.reserve(root.as_ref());
    assert!(matches!(
        denial,
        Err(CompositeHistoryCatalogDenial::MetadataCapacityExhausted {
            used: 0,
            requested,
            ..
        }) if requested == exact
    ));
    assert_eq!(less_catalog.len(), 0);
    assert_eq!(less_catalog.reserved_len(), 0);
    assert_eq!(less_catalog.metadata_ledger().total_occupancy(), 0);
}

fn assert_ledger(
    ledger: super::super::HistoryMetadataLedger,
    installed: usize,
    reservation: usize,
    promised: usize,
    total: usize,
) {
    assert_eq!(ledger.installed_resident(), installed);
    assert_eq!(ledger.reservation_resident(), reservation);
    assert_eq!(ledger.promised_installation(), promised);
    assert_eq!(ledger.total_occupancy(), total);
}

fn as_u64(bytes: usize) -> u64 {
    u64::try_from(bytes).expect("test allocation charge fits u64")
}
