//! The actual native effect owner rereads media without any full-tree witness.

use super::*;
use crate::physical_runtime::LifecycleGeneration;
use std::num::NonZeroU64;
use worth_proof::TransitionOutcome;
use worth_store_physical_backend::{qualify_filesystem_media, FilesystemQualificationRequest};

#[test]
fn moved_effect_witness_alone_detects_changed_serving_frame_and_balances_reads() {
    let (directory, media, mut coordination) = fixture::coordination();
    let (effect, format) = effect_fixture();
    let paths = fixture::install_planned_nodes(directory.path(), &effect);
    let observer = coordination.certification_residency_allocations();
    let mut window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut discovery = media.bounded_discovery(64, 2 << 20).unwrap();
    let slices =
        observe_effect_nodes(&mut discovery, &mut window, &effect, 11, format, 2 << 20).unwrap();
    let charge = slices.charged_bytes();
    assert!(charge > 0);
    let fingerprint = SelectedControlMediaFingerprint::observed_head_effect(slices);
    assert!(
        fingerprint.has_no_head_walk(),
        "no full-tree witness can mask effect reread"
    );
    let origin = window.recovery_origin_generation().unwrap();
    drop(window);
    let absent = discovery.read_current_checkpoint(4096).unwrap();
    drop(discovery.finish());
    coordination.install_absent_checkpoint(absent).unwrap();
    let (residency, ownership) = coordination.into_quiescent_recovery_parts().unwrap();
    assert!(ownership.checkpoint().is_none());
    let first = LifecycleGeneration::from_reopened(NonZeroU64::new(1).unwrap());
    let generation = if origin == first {
        LifecycleGeneration::from_reopened(NonZeroU64::new(2).unwrap())
    } else {
        first
    };
    let mut serving = PhysicalRecoveryReadAllocation::for_serving(&residency, generation);
    let TransitionOutcome::Success(media) =
        qualify_filesystem_media(FilesystemQualificationRequest::production(
            directory.path(),
            worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual Serving observation media must qualify")
    };
    let scope = Dimension::OperationScope(Scope::Recovery);
    let before = observer.snapshot().for_dimension(scope);
    assert_eq!(before.active_units(), charge);
    assert!(fingerprint
        .verify_funded_heads_for_serving(&media, format, &mut serving)
        .unwrap());
    let path = paths.first().unwrap();
    let original = std::fs::read(path).unwrap();
    let mut changed = original.clone();
    let offset = changed.len() / 2;
    changed[offset] ^= 0x40;
    std::fs::write(path, &changed).unwrap();
    assert!(!fingerprint
        .verify_funded_heads_for_serving(&media, format, &mut serving)
        .unwrap());
    assert_eq!(std::fs::read(path).unwrap(), changed);
    std::fs::write(path, original).unwrap();
    assert!(fingerprint
        .verify_funded_heads_for_serving(&media, format, &mut serving)
        .unwrap());
    let after = observer.snapshot().for_dimension(scope);
    assert_eq!(after.active_units(), charge);
    assert!(
        after.admitted_units() > before.admitted_units(),
        "real native reads must occur"
    );
    assert_eq!(
        after.admitted_units() - before.admitted_units(),
        after.released_units() - before.released_units()
    );
    drop(serving);
    drop(residency);
    assert_eq!(
        observer.snapshot().for_dimension(scope).active_units(),
        charge
    );
    drop(fingerprint);
    let disposed = observer.snapshot().for_dimension(scope);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}
