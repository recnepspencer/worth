//! Error preparation, name-copy refusal and actual backend error ownership.

use super::*;
use crate::physical_runtime::{
    certification::{MediaFaultDirective, MediaOperationRole},
    ArtifactTreeFailureKind,
};

#[test]
fn context_admission_denies_before_copy_and_payload_then_same_owner_retries() {
    let (gate, schedule) = staged_pressure::listing_pause();
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let available = size_of::<ObservedWalArtifact>() as u64 + FIRST.len() as u64 - 1;
    let mut discovery = media.bounded_discovery(4, 4096).unwrap();
    let (result, held) = staged_pressure::read_under_pressure(
        &mut discovery,
        &mut coordination,
        &gate,
        1,
        available,
    );
    let failure = result.unwrap_err();
    assert!(
        matches!(failure.diagnostic(), RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        requested,
        cause: PhysicalRecoveryObservationAllocationDenial::Residency(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }), ..
    } if requested == FIRST.len() && *required == ORIGINAL + 1 && *admitted == ORIGINAL)
    );
    assert_eq!(failure.charged_bytes(), 0);
    assert_eq!(failure.clone().charged_bytes(), 0);
    assert_eq!(discovery.counters().directory_entries_observed, 1);
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        held.bytes()
    );
    drop(held);
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let observed = window
        .read_wal_payloads(&mut discovery, segments(1), ReadGrant::ceiling_only())
        .map_err(GrantedReadStop::unread)
        .unwrap();
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    drop(observed);
    drop(window);
    drop(coordination);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

#[test]
fn actual_media_read_failure_retains_name_and_cause_until_last_clone() {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    // Qualification reads the ownership lease and persisted namespace identity;
    // the third positioned read is the real WAL payload. The named error below
    // distinguishes this boundary from an earlier setup fault.
    let schedule = authority
        .schedule(vec![authority.rule(
            MediaOperationRole::PositionedRead,
            3,
            MediaFaultDirective::FailBefore {
                kind: std::io::ErrorKind::PermissionDenied,
                raw_os_error: None,
            },
        )])
        .unwrap();
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let mut discovery = media.bounded_discovery(4, 4096).unwrap();
    let mut window = coordination.begin_source_read_allocation().unwrap();
    let failure = window
        .read_wal_payloads(&mut discovery, segments(1), ReadGrant::ceiling_only())
        .map_err(GrantedReadStop::unread)
        .unwrap_err();
    let RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Media {
        artifact: RecoveryWalArtifactView::WalArtifact(name),
        failure: cause,
    }) = failure.diagnostic()
    else {
        panic!("actual named backend denial required: {failure:?}");
    };
    assert_eq!(name, FIRST);
    assert_eq!(cause.kind(), ArtifactTreeFailureKind::DeniedBeforeEffect);
    assert_eq!(cause.io_kind(), Some(std::io::ErrorKind::PermissionDenied));
    assert_eq!(discovery.counters().bytes_read, 0);
    let retained = failure.charged_bytes();
    assert_eq!(
        retained,
        diagnostic_backing::storage_bytes() as u64 + FIRST.len() as u64
    );
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        retained
    );
    let clone = failure.clone();
    assert_eq!(clone.diagnostic(), failure.diagnostic());
    drop(failure);
    let observed = window
        .read_wal_payloads(&mut discovery, segments(1), ReadGrant::ceiling_only())
        .map_err(GrantedReadStop::unread)
        .unwrap();
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    drop(observed);
    drop(window);
    drop(coordination);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        retained
    );
    assert!(matches!(
        clone.diagnostic(),
        RecoveryWalReadFailureView::Discovery(RecoveryWalDiscoveryFailureView::Media { .. })
    ));
    drop(clone);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        0
    );
    assert_eq!(std::fs::read(wal.join(FIRST)).unwrap(), [17; PAYLOAD]);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}
