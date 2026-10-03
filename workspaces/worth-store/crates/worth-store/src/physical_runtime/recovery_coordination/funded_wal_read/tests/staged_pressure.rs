//! Apply native competition after listing or after file metadata/open settlement.

use super::*;
use crate::physical_runtime::{
    certification::{MediaFaultDirective, MediaFaultSchedule, MediaOperationRole, MediaPauseGate},
    ArtifactTreeListingAllocationBoundary, BoundedRecoveryFilesystemDiscovery,
};
use std::time::{Duration, Instant};

pub(super) fn listing_pause() -> (MediaPauseGate, MediaFaultSchedule) {
    // Existing-root qualification lists root and namespace before actual WAL.
    boundary_pause(MediaOperationRole::ListDirectory, 3)
}

pub(super) fn second_payload_pause() -> (MediaPauseGate, MediaFaultSchedule) {
    // Qualification reads identity metadata once; the two WAL lengths follow.
    boundary_pause(MediaOperationRole::ReadMetadata, 3)
}

fn boundary_pause(role: MediaOperationRole, ordinal: u64) -> (MediaPauseGate, MediaFaultSchedule) {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            role,
            ordinal,
            MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    (gate, schedule)
}

pub(super) fn read_under_pressure(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut PhysicalRecoveryCoordination,
    gate: &MediaPauseGate,
    maximum_segments: u64,
    remaining: u64,
) -> (
    Result<FundedRecoveryWalObservations, FundedRecoveryWalReadFailure>,
    OperationAllocationGrant,
) {
    read_after_boundary_pressure(
        discovery,
        coordination,
        gate,
        maximum_segments,
        remaining,
        MediaOperationRole::ListDirectory,
        3,
    )
}

pub(super) fn read_after_boundary_pressure(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut PhysicalRecoveryCoordination,
    gate: &MediaPauseGate,
    maximum_segments: u64,
    remaining: u64,
    role: MediaOperationRole,
    ordinal: u64,
) -> (
    Result<FundedRecoveryWalObservations, FundedRecoveryWalReadFailure>,
    OperationAllocationGrant,
) {
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            coordination
                .begin_source_read_allocation()
                .unwrap()
                .read_wal_payloads(discovery, maximum_segments, 4096)
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while gate.reached_context().is_none() {
            if worker.is_finished() || Instant::now() >= deadline {
                gate.release();
                panic!("actual media pause was not reached: {:?}", worker.join());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let active = observer
            .snapshot()
            .for_dimension(Dimension::OperationScope(Scope::Recovery))
            .active_units();
        let amount = ORIGINAL
            .checked_sub(active)
            .and_then(|bytes| bytes.checked_sub(remaining))
            .and_then(NonZeroU64::new);
        let held = amount.map(|bytes| ports.begin_operation(Scope::Recovery, bytes));
        // Always unblock the actual media worker before checking fixture facts.
        gate.release();
        let result = worker.join().unwrap();
        let context = gate.reached_context().unwrap();
        assert_eq!(context.role(), role);
        assert_eq!(context.role_ordinal(), ordinal);
        (
            result,
            held.expect("fixture has competitor headroom").unwrap(),
        )
    })
}

#[test]
fn tiny_original_ceiling_denies_provider_before_entries_then_adequate_owner_reads() {
    // Funds the directory path but not the actual path's iterator storage.
    let maximum = 1 << 10;
    let (directory, media, mut coordination) = world_with_capacity(maximum, None);
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let mut discovery = media.bounded_discovery(4, 4096).unwrap();
    let failure = coordination
        .begin_source_read_allocation()
        .unwrap()
        .read_wal_payloads(&mut discovery, 1, 4096)
        .unwrap_err();
    assert!(
        matches!(failure.diagnostic(), RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        cause: PhysicalRecoveryObservationAllocationDenial::ListingResidency {
            boundary: ArtifactTreeListingAllocationBoundary::ProviderIterator,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
        }, ..
    } if *required > maximum && *admitted == maximum)
    );
    assert_eq!(discovery.counters().directory_entries_observed, 0);
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(Dimension::OperationScope(Scope::Recovery))
            .active_units(),
        0
    );
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    drop(failure);
    let media = discovery.finish();
    assert_eq!(media.recovery_effect_count(), 0);
    drop(media);
    drop(coordination);
    drop(ports);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(Dimension::TotalBytes)
            .active_units(),
        0
    );
    // The immutable original ceiling cannot be widened in place. Rejoin the
    // same unchanged media with a genuinely sufficient admitted owner instead.
    let qualified =
        QualifiedRecoveryFilesystemMedia::qualify_existing(directory.path().join("store")).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let mut owner = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(ORIGINAL)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    let observer = owner.certification_residency_allocations();
    let mut discovery = media.bounded_discovery(4, 4096).unwrap();
    let observed = owner
        .begin_source_read_allocation()
        .unwrap()
        .read_wal_payloads(&mut discovery, 1, 4096)
        .unwrap();
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    drop(owner);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(Dimension::OperationScope(Scope::Recovery))
            .active_units(),
        observed.charged_bytes()
    );
    drop(observed);
    assert_eq!(
        observer
            .snapshot()
            .for_dimension(Dimension::TotalBytes)
            .active_units(),
        0
    );
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}
