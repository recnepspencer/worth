//! Two real C4 files expose result-roster and cumulative payload funding. Their arbitrary bytes
//! assert no canonical WAL meaning, integrity admission, or replay authority.
//! Qualified listing backing and original names share the real Recovery pool.

use super::*;
use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use std::{mem::size_of, num::NonZeroU64};
use worth_proof::TransitionOutcome;
use worth_store_buffer_pool::{
    PhysicalOperationAllocationScope as Scope, PhysicalResidencyDimension as Dimension,
};

const ORIGINAL: u64 = 2 << 20;
const PAYLOAD: usize = 64;
const FIRST: &str = "segment-1-generation-1.wal";
const SECOND: &str = "segment-2-generation-1.wal";

mod diagnostic_failures;
mod path_pressure;
mod serving_pressure;
mod staged_pressure;

#[test]
fn second_wal_payload_denial_retains_diagnostic_across_clone_and_owner_disposal() {
    let (gate, schedule) = staged_pressure::second_payload_pause();
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    std::fs::write(wal.join(SECOND), [29; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let roster_bytes = (2 * size_of::<ObservedWalArtifact>()) as u64;
    let diagnostic_bytes = diagnostic_backing::storage_bytes() as u64 + FIRST.len() as u64;
    // One file fits, two do not. Equal sizes avoid relying on directory order.
    let mut discovery = media.bounded_discovery(8, 4096).unwrap();
    let (result, held) = staged_pressure::read_after_boundary_pressure(
        &mut discovery,
        &mut coordination,
        &gate,
        2,
        PAYLOAD as u64 - 1,
        crate::physical_runtime::certification::MediaOperationRole::ReadMetadata,
        3,
    );
    let held_bytes = held.bytes();
    let failure = result.unwrap_err();
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalArtifact(name),
        offset,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::Residency(
                PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            ),
    } = failure.diagnostic()
    else {
        panic!("second real payload must fail the native allocation callback");
    };
    assert!(name == FIRST || name == SECOND);
    assert_eq!(offset, 0);
    assert_eq!(requested, PAYLOAD);
    assert_eq!(*required, ORIGINAL + 1);
    assert_eq!(*admitted, ORIGINAL);
    assert_eq!(
        discovery.counters().wal_bytes_read,
        PAYLOAD as u64,
        "the first file was read; the refused second payload was not"
    );
    assert_eq!(discovery.counters().bytes_read, PAYLOAD as u64);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        held_bytes + diagnostic_bytes,
        "source buffers die but the named diagnostic remains funded"
    );
    assert_eq!(failure.charged_bytes(), diagnostic_bytes);
    let retained_failure = failure.clone();
    assert_eq!(retained_failure.diagnostic(), failure.diagnostic());
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        held_bytes + diagnostic_bytes,
        "cloning shares the same funded diagnostic"
    );
    drop(failure);
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    assert_eq!(std::fs::read(wal.join(SECOND)).unwrap(), [29; PAYLOAD]);

    drop(held);
    let read_before = discovery.counters().wal_bytes_read;
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let observations = window.read_wal_payloads(&mut discovery, 2, 4096).unwrap();
    assert_eq!(observations.artifacts().len(), 2);
    for artifact in observations.artifacts() {
        let expected = if artifact.name() == FIRST {
            [17; PAYLOAD]
        } else {
            assert_eq!(artifact.name(), SECOND);
            [29; PAYLOAD]
        };
        assert_eq!(artifact.bytes(), Some(&expected[..]));
    }
    let both_bytes = 2 * PAYLOAD as u64;
    let names = observations
        .artifacts()
        .iter()
        .map(|artifact| artifact.name_heap_bytes() as u64)
        .sum::<u64>();
    assert_eq!(
        discovery.counters().wal_bytes_read - read_before,
        both_bytes
    );
    assert_eq!(
        observations.charged_bytes(),
        roster_bytes + both_bytes + names
    );
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        roster_bytes + both_bytes + names + diagnostic_bytes
    );
    drop(window);
    drop(coordination);
    assert_eq!(observations.artifacts()[0].bytes().unwrap().len(), PAYLOAD);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        roster_bytes + both_bytes + names + diagnostic_bytes,
        "roster/payload ownership survives the temporary window and Coordination owner"
    );
    drop(observations);
    assert!(matches!(
        retained_failure.diagnostic(),
        RecoveryWalReadFailureView::Allocation {
            artifact: RecoveryWalArtifactView::WalArtifact(_),
            ..
        }
    ));
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        diagnostic_bytes
    );
    drop(retained_failure);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    let media = discovery.finish();
    assert_eq!(media.recovery_effect_count(), 0);
    drop(media);
    drop(ports);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

#[test]
fn wal_result_roster_denies_before_allocation_or_payload_reads_then_retries() {
    let (gate, schedule) = staged_pressure::listing_pause();
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    std::fs::write(wal.join(SECOND), [29; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let roster_bytes = 2 * size_of::<ObservedWalArtifact>();
    let mut discovery = media.bounded_discovery(8, 4096).unwrap();
    let (result, held) = staged_pressure::read_under_pressure(
        &mut discovery,
        &mut coordination,
        &gate,
        2,
        roster_bytes as u64 - 1,
    );
    let held_bytes = held.bytes();
    let failure = result.unwrap_err();
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        offset,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::Residency(
                PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            ),
    } = failure.diagnostic()
    else {
        panic!("actual result-roster reservation must be the denial boundary");
    };
    assert_eq!(offset, 0);
    assert_eq!(requested, roster_bytes);
    assert_eq!(*required, ORIGINAL + 1);
    assert_eq!(*admitted, ORIGINAL);
    assert_eq!(
        discovery.counters().directory_entries_observed,
        2,
        "provider directory observation legitimately precedes result-roster admission"
    );
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!(discovery.counters().bytes_read, 0);
    let denied = observer.snapshot().for_dimension(dimension);
    assert_eq!(denied.active_units(), held_bytes);
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    assert_eq!(std::fs::read(wal.join(SECOND)).unwrap(), [29; PAYLOAD]);
    drop(held);
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let observations = window.read_wal_payloads(&mut discovery, 2, 4096).unwrap();
    let expected = roster_bytes as u64
        + 2 * PAYLOAD as u64
        + observations
            .artifacts()
            .iter()
            .map(|artifact| artifact.name_heap_bytes() as u64)
            .sum::<u64>();
    assert_eq!(observations.artifacts().len(), 2);
    assert_eq!(observations.charged_bytes(), expected);
    assert_eq!(discovery.counters().wal_bytes_read, 2 * PAYLOAD as u64);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        expected
    );
    drop(observations);
    drop(window);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    let media = discovery.finish();
    assert_eq!(media.recovery_effect_count(), 0);
    drop(media);
    drop(coordination);
    drop(ports);
    let disposed = observer.snapshot().for_dimension(Dimension::TotalBytes);
    assert_eq!(disposed.active_units(), 0);
    assert_eq!(disposed.admitted_units(), disposed.released_units());
}

fn world() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    world_with_schedule(None)
}

fn world_with_schedule(
    schedule: Option<crate::physical_runtime::certification::MediaFaultSchedule>,
) -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    world_with_capacity(ORIGINAL, schedule)
}

fn world_with_capacity(
    maximum: u64,
    schedule: Option<crate::physical_runtime::certification::MediaFaultSchedule>,
) -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual Store namespace must initialize");
    };
    media.close();
    let qualified = match schedule {
        Some(schedule) => {
            QualifiedRecoveryFilesystemMedia::qualify_existing_for_certification(&root, schedule)
        }
        None => QualifiedRecoveryFilesystemMedia::qualify_existing(&root),
    }
    .unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let capacity = PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
        .unwrap()
        .with_recovery_allocation_bytes(maximum)
        .unwrap();
    let coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            capacity,
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (directory, media, coordination)
}
