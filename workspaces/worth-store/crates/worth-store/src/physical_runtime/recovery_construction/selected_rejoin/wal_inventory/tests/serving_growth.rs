//! Serving freshness rereads the selected WAL members at the lengths their
//! fingerprint declares: a member that grew is stale media, not a read failure.
use super::*;
use crate::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
};
use worth_proof::TransitionOutcome;

#[test]
fn a_selected_member_one_byte_longer_is_stale_for_serving() {
    let (root, media, coordination) = fixture::coordination();
    let (path, mut encoded) = fixture::wal(root.path());
    let mut discovery = media.bounded_discovery(64, MAX_WAL_BYTES).unwrap();
    let fingerprint = admit_complete_inventory(&mut discovery, &coordination)
        .unwrap()
        .into_fingerprint();
    // Relinquish the recovery media before admitting the Serving media owner.
    drop(discovery.finish());
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.path()).unwrap()).unwrap();
    let TransitionOutcome::Success(runtime) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("the existing namespace must admit its Serving media owner");
    };
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    assert_eq!(
        fingerprint.matches_serving_media(runtime.record_serving_media(), &mut window),
        Ok(true)
    );

    encoded.push(0);
    std::fs::write(&path, &encoded).unwrap();
    assert_eq!(
        fingerprint.matches_serving_media(runtime.record_serving_media(), &mut window),
        Ok(false)
    );
}
