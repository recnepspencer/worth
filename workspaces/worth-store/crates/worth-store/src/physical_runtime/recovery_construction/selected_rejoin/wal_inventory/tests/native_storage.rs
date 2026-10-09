//! Slot growth is an owner test: values carry no WAL or Serving authority.
use super::*;
use crate::physical_runtime::RecoveryWalAllocationDenial;
use std::{mem::size_of, num::NonZeroU64};
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

#[test]
fn roster_growth_denies_before_allocation_then_moves_and_releases_exact_capacity() {
    let (_root, media, coordination) = fixture::coordination();
    let (owner, _, _) = coordination.sampling_allocation_basis().unwrap();
    let ports = owner.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let empty = WalRoster::<u64>::empty();
    assert_eq!((empty.capacity(), empty.charged_bytes()), (0, 0));
    let initial_bytes = (2 * size_of::<u64>()) as u64;
    let blocker = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(fixture::ORIGINAL - initial_bytes + 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let denial = match WalRoster::<u64>::with_capacity(&coordination, 2) {
        Err(denial) => denial,
        Ok(_) => panic!("initial slots must be funded before Vec allocation"),
    };
    assert!(matches!(denial, RecoveryWalAllocationDenial::Backing {
        requested,
        cause: ResidentDenial::BudgetExceeded { required, admitted },
    } if requested == initial_bytes && required == fixture::ORIGINAL + 1 && admitted == fixture::ORIGINAL));
    let after = observer.snapshot().for_dimension(dimension);
    assert_eq!(after.admissions(), before.admissions());
    assert_eq!(after.admitted_units(), before.admitted_units());
    assert_eq!(after.denials(), before.denials() + 1);
    assert_eq!(fixture::active(&ports), blocker.bytes());
    drop(blocker);
    let mut roster = WalRoster::<u64>::with_capacity(&coordination, 2).unwrap();
    roster.push_reserved(17);
    roster.push_reserved(29);
    let old_pointer = roster.as_ptr();
    let old_bytes = (roster.capacity() * size_of::<u64>()) as u64;
    let next_bytes = (4 * size_of::<u64>()) as u64;
    assert_eq!(fixture::active(&ports), old_bytes);
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(fixture::ORIGINAL - old_bytes - next_bytes + 1).unwrap(),
        )
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let RecoveryWalAllocationDenial::Backing {
        requested,
        cause: ResidentDenial::BudgetExceeded { required, admitted },
    } = roster.reserve_one(&coordination).unwrap_err()
    else {
        panic!("old plus prospective slot storage must deny in the actual native pool");
    };
    assert_eq!(requested, old_bytes + next_bytes);
    assert_eq!(
        (required, admitted),
        (fixture::ORIGINAL + 1, fixture::ORIGINAL)
    );
    assert_eq!(roster.as_ptr(), old_pointer);
    assert_eq!(&*roster, &[17, 29]);
    assert_eq!(roster.charged_bytes(), old_bytes);
    let after = observer.snapshot().for_dimension(dimension);
    assert_eq!(after.admissions(), before.admissions());
    assert_eq!(after.admitted_units(), before.admitted_units());
    assert_eq!(after.denials(), before.denials() + 1);
    drop(held);
    let before = observer.snapshot().for_dimension(dimension);
    roster.reserve_one(&coordination).unwrap();
    assert_ne!(
        roster.as_ptr(),
        old_pointer,
        "the old allocation was live during preparation"
    );
    let retained = (roster.capacity() * size_of::<u64>()) as u64;
    assert_eq!(roster.capacity(), 4);
    assert_eq!(roster.len(), 2, "retained spare capacity is still charged");
    assert_eq!(roster.charged_bytes(), retained);
    assert_eq!(fixture::active(&ports), retained);
    let after = observer.snapshot().for_dimension(dimension);
    assert_eq!(after.admitted_units() - before.admitted_units(), next_bytes);
    assert_eq!(after.released_units() - before.released_units(), old_bytes);
    drop(coordination);
    drop(media);
    assert_eq!(&*roster, &[17, 29]);
    assert_eq!(fixture::active(&ports), retained);
    drop(roster);
    assert_eq!(fixture::active(&ports), 0);
    fixture::assert_balanced(&observer);
}
