use super::*;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension,
};
use worth_store_physical_backend::UnchargedRead;

mod fixture;

#[test]
fn shared_checkpoint_clones_hold_actual_backing_and_charge_after_owner_close() {
    let (_directory, media, mut coordination) = fixture::coordination();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let shared = fixture::with_assembly(media.store_identity(), |assembly| {
        coordination.admit_shared_checkpoint(assembly).unwrap()
    });
    let payload = shared
        .stream()
        .certificate_records()
        .iter()
        .map(|frame| frame.len() as u64)
        .sum::<u64>();
    let roster = std::mem::size_of_val(shared.stream().certificate_records()) as u64;
    let actual = payload
        + roster
        + std::mem::size_of::<RecoveryCheckpointStorage>() as u64
        + 2 * std::mem::size_of::<usize>() as u64;
    assert_eq!(shared.owned_heap_bytes(), Some(actual));
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        actual
    );
    assert!(shared.matches_owner(&coordination.residency));
    let retained = shared.clone();
    assert!(std::ptr::eq(shared.stream(), retained.stream()));
    drop(shared);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        actual
    );
    let mut discovery = media.bounded_discovery(1, 4096).unwrap();
    let absent =
        crate::physical_runtime::recovery_coordination::observe_checkpoint_for_test(&mut discovery)
            .unwrap();
    let media = discovery.finish();
    coordination.install_absent_checkpoint(absent).unwrap();
    let (owner, ownership) = coordination.into_quiescent_recovery_parts().unwrap();
    assert!(ownership.checkpoint().is_none());
    drop(ownership);
    assert!(owner.close().requires_inspection());
    assert_eq!(retained.stream().certificate_records().len(), 1);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        actual
    );
    drop(retained);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
    drop(ports);
    drop(media);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}

#[test]
fn native_pressure_denies_certificate_construction_without_admission_then_retries() {
    let (_directory, media, mut coordination) = fixture::coordination();
    let ports = coordination.residency.ports().clone();
    let limit = coordination
        .recovery_allocation_admission()
        .unwrap()
        .byte_limit();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(limit - 1).unwrap())
        .unwrap();
    let before = ports.allocation_events().snapshot();
    let denial = fixture::with_assembly(media.store_identity(), |assembly| {
        coordination.admit_shared_checkpoint(assembly).unwrap_err()
    });
    let SharedCheckpointAdmissionDenial::Allocation(
        PhysicalRecoveryRejoinResidentDenial::OperationAllocation(failure),
    ) = denial
    else {
        panic!("constructor must preserve native pressure");
    };
    let pressure = failure.pressure().unwrap();
    assert_eq!(pressure.scope(), Scope::Recovery);
    assert_eq!(
        pressure.dimension(),
        PhysicalResidencyDimension::OperationScope(Scope::Recovery)
    );
    assert_eq!(pressure.admitted(), limit - 1);
    assert_eq!(pressure.limit(), limit);
    assert!(pressure.requested() > 1);
    assert!(!pressure.effect_may_have_started());
    let after = ports.allocation_events().snapshot();
    for dimension in [
        PhysicalResidencyDimension::OperationScope(Scope::Recovery),
        PhysicalResidencyDimension::OperationBytes,
        PhysicalResidencyDimension::TotalBytes,
    ] {
        assert_eq!(
            after.for_dimension(dimension).admitted_units(),
            before.for_dimension(dimension).admitted_units()
        );
        assert_eq!(
            after.for_dimension(dimension).admissions(),
            before.for_dimension(dimension).admissions()
        );
    }
    drop(held);
    let shared = fixture::with_assembly(media.store_identity(), |assembly| {
        coordination.admit_shared_checkpoint(assembly).unwrap()
    });
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        shared.owned_heap_bytes().unwrap()
    );
    drop(shared);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
}

#[test]
fn shared_checkpoint_rejects_foreign_store_before_native_demand_and_distinct_pool_binding() {
    let (_directory, media, mut coordination) = fixture::coordination();
    let ports = coordination.residency.ports().clone();
    let before = ports.allocation_events().snapshot();
    let foreign = worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes(
            [141; 16],
        )
        .unwrap(),
    )
    .published_identity();
    let denial = fixture::with_assembly(foreign, |assembly| {
        coordination.admit_shared_checkpoint(assembly).unwrap_err()
    });
    assert_eq!(denial, SharedCheckpointAdmissionDenial::StoreMismatch);
    assert_eq!(ports.allocation_events().snapshot(), before);
    let shared = fixture::with_assembly(media.store_identity(), |assembly| {
        coordination.admit_shared_checkpoint(assembly).unwrap()
    });
    let other = PhysicalResidencyOwner::admit(
        media.store_identity(),
        coordination.residency.admitted_policy(),
    )
    .unwrap();
    assert!(!shared.matches_owner(&other));
    drop(shared);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
}

#[test]
fn actual_absence_is_required_and_other_absent_locator_cannot_install_it() {
    use super::super::selected_checkpoint::SelectedCheckpointInstallationDenial as Denial;
    let (_directory, media, mut coordination) = fixture::coordination();
    assert_eq!(
        coordination.require_observed_checkpoint(),
        Err(Denial::Unobserved)
    );
    let before = coordination
        .residency
        .ports()
        .allocation_events()
        .snapshot();
    let mut discovery = media.bounded_discovery(3, 8192).unwrap();
    let wrong = discovery
        .read(
            worth_store_physical_backend::ArtifactCeiling::fixed(
                worth_store_physical_backend::FixedArtifact::CurrentRootSelector,
            ),
            worth_store_physical_backend::ReadGrant::ceiling_only(),
        )
        .observed()
        .unwrap();
    assert_eq!(
        coordination.install_absent_checkpoint(wrong),
        Err(Denial::InvalidAbsence)
    );
    assert_eq!(
        coordination.require_observed_checkpoint(),
        Err(Denial::Unobserved)
    );
    let absent =
        crate::physical_runtime::recovery_coordination::observe_checkpoint_for_test(&mut discovery)
            .unwrap();
    assert!(absent.bytes().is_none());
    let repeated =
        crate::physical_runtime::recovery_coordination::observe_checkpoint_for_test(&mut discovery)
            .unwrap();
    let media = discovery.finish();
    coordination.install_absent_checkpoint(absent).unwrap();
    assert_eq!(
        coordination.install_absent_checkpoint(repeated),
        Err(Denial::AlreadyInstalled)
    );
    assert!(coordination.checkpoint().is_none());
    assert_eq!(
        coordination
            .residency
            .ports()
            .allocation_events()
            .snapshot(),
        before
    );
    let (owner, ownership) = coordination.into_quiescent_recovery_parts().unwrap();
    assert!(ownership.checkpoint().is_none());
    drop(ownership);
    assert!(!owner.close().requires_inspection());
    drop(media);

    let (_directory, _media, coordination) = fixture::coordination();
    let observer = coordination.residency.ports().allocation_events();
    assert!(coordination.into_quiescent_recovery_parts().is_none());
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(PhysicalResidencyDimension::TotalBytes)
            .active_units(),
        0
    );
}
