//! The complete inventory's own budget: a WAL one byte past its whole is
//! refused unread, with the real bytes through the crossing file, before any
//! frame is retained.
use super::*;
use crate::physical_runtime::PhysicalRecoveryRejoinResidentBoundary as Boundary;

#[test]
fn an_inventory_one_byte_past_its_budget_is_refused_before_any_effect() {
    let (root, media, coordination) = fixture::coordination();
    let (path, encoded) = fixture::wal(root.path());
    let length = encoded.len() as u64;
    let (owner, original, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let mut discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    let backing = 4096;
    let mut resident = StoreRejoinResidentLedger::for_test(original, backing, u64::MAX).unwrap();

    let denial = match admit_complete(
        &mut discovery,
        &coordination,
        SelectedWalInventoryBudget::with_whole_for_test(length - 1),
        Some(&mut resident),
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("an inventory past its budget must be refused"),
    };
    let Denial::WalBytes {
        boundary: Some(Boundary::FinalWalAdmission),
        cause: limit,
    } = denial.at_resident_boundary(Boundary::FinalWalAdmission)
    else {
        panic!("the inventory's budget refuses with its own limit");
    };
    assert_eq!(limit.dimension(), SelectedWalInventoryBound::WalBytes);
    assert_eq!((limit.observed(), limit.admitted()), (length, length - 1));
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!((resident.used(), resident.peak()), (backing, backing));
    assert_eq!(fixture::active(&ports), 0);
    assert_eq!(std::fs::read(&path).unwrap(), encoded);

    let inventory = admit_complete(
        &mut discovery,
        &coordination,
        SelectedWalInventoryBudget::with_whole_for_test(length),
        None,
    )
    .unwrap();
    assert_eq!(inventory.frames().len(), 1);
    assert_eq!(discovery.counters().wal_bytes_read, length);
    drop(inventory);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    drop(coordination);
    assert_eq!(fixture::active(&ports), 0);
    fixture::assert_balanced(&observer);
}
