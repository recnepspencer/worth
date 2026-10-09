//! Original native Recovery-pool lifetime for a decoded edge owner.

use super::*;
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use std::{cell::Cell, num::NonZeroU64, rc::Rc};
use worth_proof::TransitionOutcome;
use worth_store_buffer_pool::PhysicalOperationAllocationScope as Scope;
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

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
    let mut coordination = freshness
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
    // No C8 claim has been retained in this focused owner world. The real
    // Coordination admission installs that empty live baseline once.
    coordination.admit_rejoin_resident_bytes(0).unwrap();
    (directory, media, coordination)
}

struct DropProbe<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for DropProbe<F> {
    fn drop(&mut self) {
        self.0.take().expect("drop probe runs exactly once")();
    }
}

#[test]
fn semantic_failure_disposes_decoded_data_before_native_grant_and_restores_ledger() {
    let (_directory, _media, mut coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let baseline_native = ports.counters().active_operation_bytes_for(Scope::Recovery);
    let mut resident = StoreRejoinResidentLedger::from_coordination(&mut coordination).unwrap();
    let baseline_resident = resident.used();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let mut storage = RejoinDecodeStorage::new(&window, &mut resident);
    storage.admit_allocation(64).unwrap();
    let seen = Rc::new(Cell::new(false));
    let seen_on_drop = Rc::clone(&seen);
    let observed_ports = ports.clone();
    let decoded = RejoinDecoded::new(
        DropProbe(Some(move || {
            assert_eq!(
                observed_ports
                    .counters()
                    .active_operation_bytes_for(Scope::Recovery),
                baseline_native + 64,
                "the decoded value must drop while its native grant is live",
            );
            seen_on_drop.set(true);
        })),
        storage.into_charge(),
    );
    let failure = decoded.with_data(&mut resident, |_data, _resident| {
        Err::<(), Denial>(Denial::CertificateRoster)
    });
    assert!(matches!(failure, Err(Denial::CertificateRoster)));
    assert!(seen.get());
    assert_eq!(resident.used(), baseline_resident);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        baseline_native
    );
}

#[test]
fn native_pressure_denies_decoder_request_without_retained_charge() {
    let (_directory, _media, mut coordination) = recovery_world();
    let ports = coordination
        .sampling_allocation_basis()
        .unwrap()
        .0
        .ports()
        .clone();
    let mut resident = StoreRejoinResidentLedger::from_coordination(&mut coordination).unwrap();
    let baseline_resident = resident.used();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let baseline_native = ports.counters().active_operation_bytes_for(Scope::Recovery);
    let available = window
        .recovery_byte_limit()
        .checked_sub(baseline_native)
        .unwrap();
    let held = ports
        .begin_operation(Scope::Recovery, NonZeroU64::new(available - 1).unwrap())
        .unwrap();
    let mut storage = RejoinDecodeStorage::new(&window, &mut resident);
    assert!(matches!(
        storage.admit_allocation(2),
        Err(Denial::Resident(_))
    ));
    drop(storage);
    assert_eq!(resident.used(), baseline_resident);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        baseline_native + held.bytes(),
    );
    drop(held);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        baseline_native
    );
}
