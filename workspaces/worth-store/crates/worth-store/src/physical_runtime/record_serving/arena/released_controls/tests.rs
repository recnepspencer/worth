use super::*;
use crate::physical_runtime::record_serving::arena::{
    ExtentArenaAllocationOwner, ExtentArenaCapacity,
};
use std::sync::{Arc, Mutex};
use worth_store_physical_format::{ExtentArenaId, PhysicalRecordFormatDeclaration};

fn owner(maximum_ranges: usize) -> SharedArenaAllocationOwner {
    Arc::new(Mutex::new(
        ExtentArenaAllocationOwner::new(ExtentArenaCapacity::DEFAULT, 4096, maximum_ranges, 2)
            .unwrap(),
    ))
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn whole_first_arena() -> ExtentArenaRange {
    ExtentArenaRange::new(
        ExtentArenaId::new(1).unwrap(),
        0,
        ExtentArenaCapacity::DEFAULT.get(),
    )
    .unwrap()
}

#[test]
fn three_controls_acquire_exact_named_claims_before_competing_allocation() {
    let owner = owner(8);
    owner
        .lock()
        .unwrap()
        .restore_free_range(whole_first_arena())
        .unwrap();
    let attempt = [7; 16];
    let manifest_bytes = 600_u64;
    let bundle =
        ReleasedControlArenaReservations::reserve(&owner, attempt, format(), manifest_bytes, None)
            .unwrap();
    let (manifest, reservation, descriptor) = bundle.into_parts();
    let controls = [
        (&manifest, BlobRecordKind::DropSetManifestV3, manifest_bytes),
        (
            &reservation,
            BlobRecordKind::OriginalDropReserved,
            OriginalDropReservedV1::encoded_frame_bytes() as u64,
        ),
        (
            &descriptor,
            BlobRecordKind::ReclaimDescriptorV3,
            BlobReclaimDescriptorV3::encoded_frame_bytes() as u64,
        ),
    ];
    for (control, kind, bytes) in controls {
        assert!(control.matches(attempt, kind, bytes));
        assert_eq!(control.encoded_bytes(), bytes);
        assert!(control.reservation().belongs_to(&owner));
        assert!(!control.matches([8; 16], kind, bytes));
        assert!(!control.matches(attempt, kind, bytes + 1));
    }
    assert!(!manifest.matches(attempt, BlobRecordKind::ReclaimDescriptorV3, manifest_bytes));
    let ranges = [
        manifest.reservation().range(),
        reservation.reservation().range(),
        descriptor.reservation().range(),
    ];
    for (index, range) in ranges.iter().enumerate() {
        assert!(ranges[index + 1..]
            .iter()
            .all(|next| range.end() <= next.offset()));
    }
    let competing = ArenaReservation::reserve(&owner, 4096).unwrap();
    assert!(competing.range().offset() >= ranges[2].end());
}

#[test]
fn third_claim_denial_cancels_both_pre_effect_claims_under_owner_lock() {
    let owner = owner(3);
    owner
        .lock()
        .unwrap()
        .restore_free_range(whole_first_arena())
        .unwrap();
    let denial = ReleasedControlArenaReservations::reserve(&owner, [7; 16], format(), 600, None)
        .err()
        .unwrap();
    assert_eq!(
        denial,
        ArenaAllocationDenial::RangeBudget {
            required: 4,
            maximum: 3,
        }
    );
    // Reverse cancellation coalesces the original free range. The token
    // frontier may advance, but neither abandoned claim remains live.
    assert_eq!(
        owner
            .lock()
            .unwrap()
            .reserve(ExtentArenaCapacity::DEFAULT.get())
            .unwrap()
            .1,
        whole_first_arena(),
    );
}

#[test]
fn unused_claims_cancel_but_wal_exposed_claim_remains_quarantined() {
    let owner = owner(8);
    owner
        .lock()
        .unwrap()
        .restore_free_range(whole_first_arena())
        .unwrap();
    let (manifest, reservation, descriptor) =
        ReleasedControlArenaReservations::reserve(&owner, [7; 16], format(), 600, None)
            .unwrap()
            .into_parts();
    let manifest_range = manifest.reservation().range();
    let obligation = manifest.reservation().retain_obligation();
    let (exposed, _) = manifest.into_reservations();
    exposed.expose_to_wal();
    drop(exposed);
    drop(reservation);
    drop(descriptor);
    let competing = ArenaReservation::reserve(&owner, manifest_range.length()).unwrap();
    assert_ne!(competing.range(), manifest_range);
    obligation.cancel_after_resolution().unwrap();
    drop(competing);
    assert_eq!(
        ArenaReservation::reserve(&owner, manifest_range.length())
            .unwrap()
            .range(),
        manifest_range
    );
}

#[test]
fn directory_claim_rides_the_descriptor_and_is_all_or_nothing() {
    let owner = owner(8);
    owner
        .lock()
        .unwrap()
        .restore_free_range(whole_first_arena())
        .unwrap();
    let (manifest, reservation, descriptor) =
        ReleasedControlArenaReservations::reserve(&owner, [7; 16], format(), 600, Some(900))
            .unwrap()
            .into_parts();
    assert_eq!((manifest.claim_count(), reservation.claim_count()), (1, 1));
    assert_eq!(manifest.directory_encoded_bytes(), None);
    assert_eq!(descriptor.claim_count(), 2);
    assert_eq!(descriptor.directory_encoded_bytes(), Some(900));
    let (control_bytes, control) = descriptor.claim(0).unwrap();
    let (directory_bytes, directory) = descriptor.claim(1).unwrap();
    assert!(descriptor.claim(2).is_none());
    assert_eq!(control_bytes, descriptor.encoded_bytes());
    assert_eq!(directory_bytes, 900);
    assert!(directory.belongs_to(&owner));
    assert!(control.range().end() <= directory.range().offset());

    // One range short of the fourth claim: every claim is cancelled.
    let owner = owner_with_ranges(4);
    let denial =
        ReleasedControlArenaReservations::reserve(&owner, [7; 16], format(), 600, Some(900))
            .err()
            .unwrap();
    assert_eq!(
        denial,
        ArenaAllocationDenial::RangeBudget {
            required: 5,
            maximum: 4,
        }
    );
    assert_eq!(
        owner
            .lock()
            .unwrap()
            .reserve(ExtentArenaCapacity::DEFAULT.get())
            .unwrap()
            .1,
        whole_first_arena(),
    );
}

fn owner_with_ranges(maximum_ranges: usize) -> SharedArenaAllocationOwner {
    let owner = owner(maximum_ranges);
    owner
        .lock()
        .unwrap()
        .restore_free_range(whole_first_arena())
        .unwrap();
    owner
}
