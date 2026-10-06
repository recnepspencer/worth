//! Native ownership of retained addressed effect witnesses.

use super::*;
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryCoordination,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission,
    PhysicalStore, QualifiedRecoveryFilesystemMedia,
};
use std::{mem::size_of, num::NonZeroU64};
use worth_proof::TransitionOutcome;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};
use worth_store_physical_format::{PhysicalRecordFormatDeclaration, RecordArtifactFile};

#[path = "tests/merge_growth.rs"]
mod merge_growth;

fn recovery_world() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let directory = tempfile::tempdir().unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(directory.path()).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("namespace initialization must admit");
    };
    media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(directory.path()).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(2 << 20)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(AdmittedPhysicalRecordFormat::admit(
                format,
            )),
            None,
        )
        .unwrap();
    (directory, media, coordination)
}

fn slice(offset: u64) -> SelectedArtifactSlice {
    SelectedArtifactSlice::observed(
        RecordArtifactFile::CurrentRootSelector,
        offset,
        &[offset as u8],
        false,
    )
    .unwrap()
}

fn funded(window: &PhysicalRecoveryReadAllocation<'_>, offset: u64) -> FundedHeadEffectSlices {
    let mut owner = FundedHeadEffectSlices::prepare(window, 1).unwrap();
    owner.push(slice(offset)).unwrap();
    owner
}

#[test]
fn native_pool_pressure_denies_before_effect_storage_is_allocated() {
    let (_directory, _media, coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let events = ports.allocation_events();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let limit = window.recovery_byte_limit();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(limit - 1).unwrap())
        .unwrap();
    let before = events.snapshot();
    assert!(matches!(
        FundedHeadEffectSlices::prepare(&window, 1),
        Err(Denial::Resident(ResidentDenial::BudgetExceeded { admitted, .. }))
            if admitted == limit
    ));
    let after = events.snapshot();
    for dimension in [
        Dimension::OperationScope(Scope::Recovery),
        Dimension::OperationBytes,
        Dimension::TotalBytes,
    ] {
        assert_eq!(
            after.for_dimension(dimension).admitted_units(),
            before.for_dimension(dimension).admitted_units()
        );
        assert_eq!(
            after.for_dimension(dimension).active_units(),
            before.for_dimension(dimension).active_units()
        );
    }
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        held.bytes()
    );
    drop(held);
    let owner = funded(&window, 1);
    let slot = size_of::<SelectedArtifactSlice>() as u64;
    assert_eq!(owner.charged_bytes(), slot);
    assert_eq!(owner.owned_heap_bytes(), Some(slot));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        slot
    );
    drop(window);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        slot
    );
    drop(owner);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn merge_pressure_preserves_destination_and_funds_donor_until_denial() {
    let (_directory, _media, coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut destination = funded(&window, 1);
    let donor = funded(&window, 2);
    let slot = size_of::<SelectedArtifactSlice>() as u64;
    let previous_charge = destination.charged_bytes();
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        2 * slot
    );
    // Replacement needs two fresh slots while both one-slot grants remain.
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(window.recovery_byte_limit() - 4 * slot + 1).unwrap(),
        )
        .unwrap();
    assert!(matches!(
        destination.merge(donor),
        Err(Denial::Resident(ResidentDenial::OperationAllocation(_)))
    ));
    assert_eq!(destination.slices(), &[slice(1)]);
    assert_eq!(destination.charged_bytes(), previous_charge);
    assert_eq!(destination.owned_heap_bytes(), Some(previous_charge));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        held.bytes() + previous_charge,
        "the rejected donor has released its own native grant"
    );
    drop(held);
    drop(destination);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn successful_merge_has_exact_native_charge_through_owner_drop() {
    let (_directory, _media, coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut destination = funded(&window, 1);
    let donor = funded(&window, 2);
    let slot = size_of::<SelectedArtifactSlice>() as u64;
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        2 * slot
    );
    destination.merge(donor).unwrap();
    assert_eq!(destination.slices(), &[slice(1), slice(2)]);
    assert_eq!(destination.charged_bytes(), 2 * slot);
    assert_eq!(destination.owned_heap_bytes(), Some(2 * slot));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        destination.charged_bytes()
    );
    drop(window);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        2 * slot
    );
    drop(destination);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn foreign_native_owner_cannot_merge_into_destination() {
    let (_directory, _media, coordination) = recovery_world();
    let (_foreign_directory, _foreign_media, foreign) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let foreign_ports = foreign
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let foreign_window = PhysicalRecoveryReadAllocation::for_coordination(&foreign).unwrap();
    let mut destination = funded(&window, 1);
    let donor = funded(&foreign_window, 2);
    let charge = destination.charged_bytes();
    assert!(!donor.matching_owner(&window));
    assert!(matches!(destination.merge(donor), Err(Denial::RootBinding)));
    assert_eq!(destination.slices(), &[slice(1)]);
    assert_eq!(destination.charged_bytes(), charge);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        charge
    );
    assert_eq!(foreign_ports.counters().active_operation_bytes(), 0);
    drop(destination);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}

#[test]
fn fingerprint_merges_reject_foreign_effect_owner_before_raw_growth() {
    use super::super::{SelectedControlMediaFingerprint, StoreRejoinResidentLedger};

    let (_directory, _media, coordination) = recovery_world();
    let (_foreign_directory, _foreign_media, foreign) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let foreign_ports = foreign
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let foreign_window = PhysicalRecoveryReadAllocation::for_coordination(&foreign).unwrap();
    let slot = size_of::<SelectedArtifactSlice>() as u64;
    for entry_point in 0..3 {
        let mut destination = SelectedControlMediaFingerprint::observed(vec![slice(11)]);
        destination.effects = Some(funded(&window, 1));
        let mut donor = SelectedControlMediaFingerprint::observed(vec![slice(22)]);
        donor.effects = Some(funded(&foreign_window, 2));
        let raw_capacity = destination.slices.capacity();
        let charge = destination.effects.as_ref().unwrap().charged_bytes();
        assert_eq!(charge, slot);
        assert_eq!(
            ports.counters().active_operation_bytes_for(Scope::Recovery),
            slot
        );
        assert_eq!(
            foreign_ports
                .counters()
                .active_operation_bytes_for(Scope::Recovery),
            slot
        );
        let limit = window.recovery_byte_limit();
        let mut exhausted = StoreRejoinResidentLedger::from_retained_with_limit(
            PhysicalRecoveryAllocationAdmission::new(window.store_identity(), limit),
            limit,
            limit,
        )
        .unwrap();
        assert_eq!(exhausted.remaining(), 0);
        let denial = match entry_point {
            0 => destination.extend(donor),
            1 => destination.extend_with_resident(donor, &mut exhausted),
            2 => destination.try_extend_bounded(donor, 0),
            _ => unreachable!(),
        };
        assert!(matches!(denial, Err(Denial::RootBinding)));
        assert_eq!(destination.slices, vec![slice(11)]);
        assert_eq!(destination.slices.capacity(), raw_capacity);
        let effects = destination.effects.as_ref().unwrap();
        assert_eq!(effects.slices(), &[slice(1)]);
        assert_eq!(effects.charged_bytes(), charge);
        assert_eq!(effects.owned_heap_bytes(), Some(charge));
        assert_eq!(
            ports.counters().active_operation_bytes_for(Scope::Recovery),
            slot
        );
        assert_eq!(foreign_ports.counters().active_operation_bytes(), 0);
        drop(destination);
        assert_eq!(ports.counters().active_operation_bytes(), 0);
    }
}

#[test]
fn serving_observer_generation_preserves_recovery_origin_and_native_pool() {
    use crate::physical_runtime::LifecycleGeneration;

    let (_directory, media, mut coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let origin = window.recovery_origin_generation().unwrap();
    let owner = funded(&window, 1);
    let charge = owner.charged_bytes();
    drop(window);
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let absent =
        crate::physical_runtime::recovery_coordination::observe_checkpoint_for_test(&mut discovery)
            .unwrap();
    let _media = discovery.finish();
    coordination.install_absent_checkpoint(absent).unwrap();
    let (residency, ownership) = coordination.into_quiescent_recovery_parts().unwrap();
    assert!(ownership.checkpoint().is_none());
    let first = LifecycleGeneration::from_reopened(NonZeroU64::new(1).unwrap());
    let observer = if origin == first {
        LifecycleGeneration::from_reopened(NonZeroU64::new(2).unwrap())
    } else {
        first
    };
    let serving = PhysicalRecoveryReadAllocation::for_serving(&residency, observer);
    assert_ne!(serving.generation(), origin);
    assert_eq!(serving.recovery_origin_generation(), Some(origin));
    assert!(owner.matching_owner(&serving));
    drop(serving);
    drop(residency);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        charge
    );
    drop(owner);
    assert_eq!(ports.counters().active_operation_bytes(), 0);
}
