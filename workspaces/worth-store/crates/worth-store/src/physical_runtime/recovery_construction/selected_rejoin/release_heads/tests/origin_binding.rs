//! Recovery-origin binding survives the actual owner move into Serving.

use super::*;
use crate::physical_runtime::LifecycleGeneration;
use std::num::NonZeroU64;
use worth_store_buffer_pool::PhysicalOperationAllocationScope as Scope;

#[test]
fn moved_recovery_owner_matches_heads_despite_distinct_serving_observer_generation() {
    let (_directory, media, mut coordination) = fixture::coordination();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let origin = window.recovery_origin_generation().unwrap();
    let (root, frame, digest) = one_head();
    let mut numeric = resident(&window, window.recovery_byte_limit());
    let heads = observe_with_read(
        &root,
        format(),
        1,
        digest,
        window.recovery_byte_limit(),
        &window,
        &mut numeric,
        |_, _, storage| charged_frame(&frame, storage),
    )
    .unwrap()
    .into_funded_slices_with_resident(&mut numeric)
    .unwrap();
    let empty_root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .admit()
        .unwrap();
    let empty_digest = ReleaseCustodyHeadRosterDigestV1::new(None, 0).finish().1;
    let empty = observe_with_read(
        &empty_root,
        format(),
        0,
        empty_digest,
        window.recovery_byte_limit(),
        &window,
        &mut numeric,
        |_, _, _| panic!("empty root requires no read"),
    )
    .unwrap()
    .into_funded_slices_with_resident(&mut numeric)
    .unwrap();
    assert_eq!(empty.charged_bytes(), 0);
    drop(window);

    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let absent = discovery.read_current_checkpoint(4096).unwrap();
    let _media = discovery.finish();
    coordination.install_absent_checkpoint(absent).unwrap();
    let (owner, ownership) = coordination.into_quiescent_recovery_parts().unwrap();
    assert!(ownership.checkpoint().is_none());
    let first_generation = LifecycleGeneration::from_reopened(NonZeroU64::new(1).unwrap());
    let observer_generation = if origin == first_generation {
        LifecycleGeneration::from_reopened(NonZeroU64::new(2).unwrap())
    } else {
        first_generation
    };
    let serving = PhysicalRecoveryReadAllocation::for_serving(&owner, observer_generation);
    assert_ne!(serving.generation(), origin);
    assert_eq!(serving.recovery_origin_generation(), Some(origin));
    assert!(heads.matching_owner(&serving));
    assert!(empty.matching_owner(&serving));
    let (_foreign_directory, _foreign_media, foreign) = fixture::coordination();
    let foreign_window = PhysicalRecoveryReadAllocation::for_coordination(&foreign).unwrap();
    assert!(!heads.matching_owner(&foreign_window));
    assert!(!empty.matching_owner(&foreign_window));
    drop(serving);
    let bytes = heads.charged_bytes();
    drop(owner);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        bytes
    );
    drop((heads, empty));
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}
