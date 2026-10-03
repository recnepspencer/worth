//! Store-entry WAL accounting: the whole-WAL read is charged on top of live
//! Store-entry backing. A real open reaches C8 planning's higher peak under the
//! same recovery-memory setting first, so this denial is owned and tested here.
use super::*;

#[test]
fn store_entry_wal_read_denies_below_backing_plus_wal_charge() {
    let (root, media, coordination) = fixture::coordination();
    let (path, encoded) = fixture::wal(root.path());
    let (owner, original, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let mut discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    let wal_charge = PhysicalRecoveryReadAllocation::for_coordination(&coordination)
        .unwrap()
        .read_wal_payloads(&mut discovery, MAX_WAL_SEGMENTS, MAX_WAL_BYTES)
        .unwrap()
        .charged_bytes();
    assert!(wal_charge >= encoded.len() as u64);
    assert_eq!(fixture::active(&ports), 0);

    let backing = 4096;
    let tight = backing + wal_charge - 1;
    let mut resident =
        StoreRejoinResidentLedger::for_test(original, backing, tight - backing).unwrap();
    let denial = match admit_complete_inventory_with_resident(
        &mut discovery,
        &coordination,
        &mut resident,
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("a ceiling below backing plus the WAL charge must deny"),
    };
    assert!(matches!(
        denial,
        Denial::Resident(ResidentDenial::BudgetExceeded { required, admitted })
            if required == backing + wal_charge && admitted == tight
    ));
    assert_eq!((resident.used(), resident.peak()), (backing, backing));
    assert_eq!(fixture::active(&ports), 0);
    assert_eq!(std::fs::read(&path).unwrap(), encoded);

    let mut sufficient = StoreRejoinResidentLedger::for_test(original, backing, u64::MAX).unwrap();
    let inventory =
        admit_complete_inventory_with_resident(&mut discovery, &coordination, &mut sufficient)
            .unwrap();
    assert_eq!(inventory.frames().len(), 1);
    assert!(sufficient.peak() >= backing + wal_charge);
    drop(inventory);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
    drop(coordination);
    assert_eq!(fixture::active(&ports), 0);
    fixture::assert_balanced(&observer);
}
