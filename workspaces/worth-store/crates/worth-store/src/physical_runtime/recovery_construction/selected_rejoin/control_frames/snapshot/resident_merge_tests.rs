use super::*;
use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};

fn ledger(bytes: u64) -> StoreRejoinResidentLedger {
    // Only allocation arithmetic is under test; this identity opens no media.
    let identity = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([1; 16]).unwrap(),
    )
    .published_identity();
    StoreRejoinResidentLedger::for_test(
        PhysicalRecoveryAllocationAdmission::new(identity, bytes),
        0,
        u64::MAX,
    )
    .unwrap()
}

fn charged_fingerprint(
    resident: &mut StoreRejoinResidentLedger,
    offset: u64,
) -> SelectedControlMediaFingerprint {
    let mut slices = resident.reserve_vec(1).unwrap();
    slices.push(
        SelectedArtifactSlice::observed(
            RecordArtifactFile::CurrentRootSelector,
            offset,
            &[offset as u8],
            false,
        )
        .unwrap(),
    );
    SelectedControlMediaFingerprint::observed(slices)
}

#[test]
fn merge_admits_donor_and_old_new_destination_overlap_before_growth() {
    let slot = std::mem::size_of::<SelectedArtifactSlice>() as u64;
    // Final backing needs two slots, but old destination + donor + new
    // destination needs four while the allocator replaces the destination.
    let mut insufficient = ledger(3 * slot);
    let mut destination = charged_fingerprint(&mut insufficient, 0);
    let donor = charged_fingerprint(&mut insufficient, 1);
    let old_capacity = destination.slices.capacity();
    assert_eq!(
        destination.extend_with_resident(donor, &mut insufficient),
        Err(PhysicalRecoveryRejoinResidentDenial::BudgetExceeded {
            required: 4 * slot,
            admitted: 3 * slot,
        })
    );
    assert_eq!(destination.slices.len(), 1);
    assert_eq!(destination.slices.capacity(), old_capacity);
    assert_eq!(insufficient.peak(), 2 * slot);

    let mut sufficient = ledger(4 * slot);
    let mut destination = charged_fingerprint(&mut sufficient, 0);
    let donor = charged_fingerprint(&mut sufficient, 1);
    destination
        .extend_with_resident(donor, &mut sufficient)
        .unwrap();
    assert_eq!(destination.slices.len(), 2);
    assert_eq!(destination.slices[0].offset, 0);
    assert_eq!(destination.slices[1].offset, 1);
    let actual = sufficient.vector_bytes(&destination.slices).unwrap();
    assert_eq!(sufficient.used(), actual);
    assert_eq!(sufficient.peak(), 4 * slot);
    drop(destination);
    sufficient.release(actual);
    assert_eq!(sufficient.used(), 0);
}
