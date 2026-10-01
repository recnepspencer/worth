use super::*;
use crate::physical_runtime::durability::retention::PhysicalRetentionProfile;
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

fn admission() -> Arc<PhysicalPublicationAdmission> {
    Arc::new(PhysicalPublicationAdmission::new(
        PhysicalRetentionProfile::new(10_000, 10, 1_000, 1).unwrap(),
    ))
}

fn reservation(segment: u64, start: u64, end: u64) -> WalPublicationReservation {
    WalPublicationReservation::new(
        (segment, 1),
        WalLsnRange::new(LogSequenceNumber::new(start), LogSequenceNumber::new(end)).unwrap(),
        100,
        1_000,
    )
    .unwrap()
}

#[test]
fn settlement_refunds_only_exact_group_once_and_reclamation_releases_current_charge() {
    let owner = admission();
    owner
        .reserve_wal_publication(reservation(1, 1, 3))
        .unwrap()
        .seal();
    owner
        .reserve_wal_publication(reservation(1, 3, 4))
        .unwrap()
        .seal();
    owner
        .reserve_wal_publication(reservation(2, 4, 5))
        .unwrap()
        .seal();
    assert_eq!(owner.charged_growth_bytes(), 3_300);
    for (end, wal, metadata) in [(2, 100, 200), (3, 99, 200), (3, 100, 1_001)] {
        assert!(owner.settle_exact_group(1, end, wal, metadata).is_err());
        assert_eq!(owner.charged_growth_bytes(), 3_300);
    }
    owner.settle_exact_group(1, 3, 100, 200).unwrap();
    assert_eq!(owner.charged_growth_bytes(), 2_500);
    assert!(owner.settle_exact_group(1, 3, 100, 1).is_err());
    assert_eq!(owner.charged_growth_bytes(), 2_500);
    owner.release_sealed_publication(1, 1);
    assert_eq!(owner.charged_growth_bytes(), 1_100);
    owner.release_sealed_publication(1, 1);
    assert_eq!(owner.charged_growth_bytes(), 1_100);
    owner.release_sealed_publication(2, 1);
    assert_eq!(owner.charged_growth_bytes(), 0);
}

#[test]
fn no_effect_releases_registration_but_uncertain_effect_keeps_full_reservation() {
    let owner = admission();
    let charge = owner.reserve_wal_publication(reservation(1, 1, 3)).unwrap();
    assert!(owner.reserve_wal_publication(reservation(1, 1, 3)).is_err());
    assert_eq!(owner.charged_growth_bytes(), 1_100);
    drop(charge);
    assert_eq!(owner.charged_growth_bytes(), 0);
    owner
        .reserve_wal_publication(reservation(1, 1, 3))
        .unwrap()
        .seal();
    assert_eq!(owner.charged_growth_bytes(), 1_100);
    assert!(owner.settle_exact_group(1, 2, 50, 200).is_err());
    assert_eq!(owner.charged_growth_bytes(), 1_100);
}

#[test]
fn growth_denial_leaves_no_group_registration_or_partial_charge() {
    let owner = admission();
    owner
        .reserve_wal_publication(reservation(1, 1, 3))
        .unwrap()
        .seal();
    let mut over_budget = reservation(2, 3, 5);
    // The profile keeps 1,000 bytes for maintenance, leaving 7,900 after
    // the first 1,100-byte group reservation.
    over_budget.metadata_bytes = 7_801;
    let denial = match owner.reserve_wal_publication(over_budget) {
        Err(denial) => denial,
        Ok(_) => panic!("one byte over the remaining grant must deny"),
    };
    assert_eq!(denial.requested_bytes, 7_901);
    assert_eq!(denial.remaining_bytes, 7_900);
    assert_eq!(owner.charged_growth_bytes(), 1_100);
    let retry = owner.reserve_wal_publication(reservation(2, 3, 5)).unwrap();
    assert_eq!(owner.charged_growth_bytes(), 2_200);
    drop(retry);
    assert_eq!(owner.charged_growth_bytes(), 1_100);
}

#[test]
fn reopened_groups_restore_settled_and_unresolved_charges_without_double_refund() {
    let owner = admission();
    owner
        .restore_wal_publication(reservation(1, 1, 3), Some(200))
        .unwrap();
    owner
        .restore_wal_publication(reservation(1, 3, 4), None)
        .unwrap();
    assert_eq!(owner.charged_growth_bytes(), 1_400);
    assert!(owner
        .restore_wal_publication(reservation(1, 1, 3), Some(200))
        .is_err());
    assert!(owner
        .restore_wal_publication(reservation(2, 4, 5), Some(1_001))
        .is_err());
    assert!(owner.settle_exact_group(1, 3, 100, 100).is_err());
    assert_eq!(owner.charged_growth_bytes(), 1_400);
    owner.release_sealed_publication(1, 1);
    assert_eq!(owner.charged_growth_bytes(), 0);
}
