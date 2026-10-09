//! Actual C4/native pressure proves mechanical constraints, not a synthetic C8 seal.

use super::*;
use crate::physical_runtime::certification::{
    MediaFaultDirective, MediaFaultSchedule, MediaOperationRole, MediaPauseGate,
};
use std::{
    ffi::OsStr,
    time::{Duration, Instant},
};
use worth_store_physical_backend::{
    ArtifactTreeDirectoryEntry, BoundedRecoveryFilesystemDiscovery,
    RecoverySelectedWalReadOutcome as Outcome, RecoveryWalReadSelection,
    RecoveryWalSelectionMismatch as Mismatch,
};
use worth_store_physical_format::store_namespace::NamespaceEntryType;

struct SingleFile;
impl RecoveryWalReadSelection for SingleFile {
    fn matches_listing(&self, entries: &mut [ArtifactTreeDirectoryEntry]) -> bool {
        matches!(entries, [entry] if entry.name() == FIRST && entry.entry_type() == NamespaceEntryType::RegularFile)
    }
    fn expected_file_length(&self, name: &OsStr) -> Option<u64> {
        (name == FIRST).then_some(PAYLOAD as u64)
    }
}

#[test]
fn listing_count_unknown_name_and_type_drift_precede_roster_admission_under_pressure() {
    for drift in 0..3 {
        let (gate, schedule) = pause(MediaOperationRole::ListDirectory, 3);
        let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
        let wal = directory.path().join("store/families/wal");
        std::fs::create_dir_all(&wal).unwrap();
        match drift {
            0 => {
                std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
                std::fs::write(wal.join(SECOND), [29; PAYLOAD]).unwrap();
            }
            1 => std::fs::write(wal.join(SECOND), [29; PAYLOAD]).unwrap(),
            2 => std::fs::create_dir(wal.join(FIRST)).unwrap(),
            _ => unreachable!(),
        }
        let ports = coordination.residency.ports().clone();
        let mut discovery = media.bounded_discovery(8, 4096).unwrap();
        let (result, held) = read_at_saturated_boundary(
            &mut coordination,
            &mut discovery,
            &gate,
            MediaOperationRole::ListDirectory,
            3,
        );
        assert!(matches!(
            result.unwrap(),
            Outcome::Mismatch(Mismatch::Listing)
        ));
        assert_eq!(discovery.counters().wal_bytes_read, 0);
        assert_eq!(discovery.counters().bytes_read, 0);
        assert_eq!(active(&ports), held.bytes());
        drop(held);
        assert_eq!(active(&ports), 0);
        assert_eq!(discovery.finish().recovery_effect_count(), 0);
    }
}

#[test]
fn enlarged_and_shortened_lengths_precede_payload_admission_under_pressure() {
    for length in [PAYLOAD - 1, PAYLOAD + 1] {
        let (gate, schedule) = pause(MediaOperationRole::ReadMetadata, 2);
        let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
        let wal = directory.path().join("store/families/wal");
        std::fs::create_dir_all(&wal).unwrap();
        std::fs::write(wal.join(FIRST), vec![17; length]).unwrap();
        let ports = coordination.residency.ports().clone();
        let mut discovery = media.bounded_discovery(8, 4096).unwrap();
        let (result, held) = read_at_saturated_boundary(
            &mut coordination,
            &mut discovery,
            &gate,
            MediaOperationRole::ReadMetadata,
            2,
        );
        if length < PAYLOAD {
            assert!(
                matches!(result.unwrap(), Outcome::Mismatch(Mismatch::FileLength {
                expected, observed,
            }) if expected == PAYLOAD as u64 && observed == length as u64)
            );
        } else {
            // Past its declared length the member is damage, with its real
            // length: the selection's drift is only a shorter member.
            let failure = result.unwrap_err();
            assert!(
                matches!(failure.diagnostic(), RecoveryWalReadFailureView::Discovery(
                RecoveryWalDiscoveryFailureView::PastCeiling {
                    artifact: RecoveryWalArtifactView::WalArtifact(name), length: real, ceiling,
                }) if name == FIRST && real == length as u64 && ceiling == PAYLOAD as u64)
            );
        }
        assert_eq!(discovery.counters().wal_bytes_read, 0);
        assert_eq!(discovery.counters().bytes_read, 0);
        assert_eq!(active(&ports), held.bytes());
        drop(held);
        std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
        let window = coordination.begin_source_read_allocation().unwrap();
        let Outcome::Observed(observed) = selected_read(&window, &mut discovery).unwrap() else {
            panic!("restored exact file must match the same mechanical selection");
        };
        assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
        assert_eq!(active(&ports), observed.charged_bytes());
        drop(observed);
        drop(window);
        assert_eq!(active(&ports), 0);
        assert_eq!(discovery.finish().recovery_effect_count(), 0);
    }
}

#[test]
fn healthy_selected_length_preserves_native_payload_denial_then_retry() {
    let (gate, schedule) = pause(MediaOperationRole::ReadMetadata, 2);
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    let wal = directory.path().join("store/families/wal");
    std::fs::create_dir_all(&wal).unwrap();
    std::fs::write(wal.join(FIRST), [17; PAYLOAD]).unwrap();
    let ports = coordination.residency.ports().clone();
    let mut discovery = media.bounded_discovery(8, 4096).unwrap();
    let (result, held) = read_at_saturated_boundary(
        &mut coordination,
        &mut discovery,
        &gate,
        MediaOperationRole::ReadMetadata,
        2,
    );
    let failure = result.unwrap_err();
    assert!(
        matches!(failure.diagnostic(), RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalArtifact(name), requested,
        cause: PhysicalRecoveryObservationAllocationDenial::Residency(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }), ..
    } if name == FIRST && requested == PAYLOAD && *required == ORIGINAL + PAYLOAD as u64 && *admitted == ORIGINAL)
    );
    assert_eq!(discovery.counters().wal_bytes_read, 0);
    assert_eq!(active(&ports), held.bytes() + failure.charged_bytes());
    drop(failure);
    drop(held);
    let window = coordination.begin_source_read_allocation().unwrap();
    let Outcome::Observed(observed) = selected_read(&window, &mut discovery).unwrap() else {
        panic!("native release must make the unchanged healthy file readable");
    };
    assert_eq!(observed.artifacts()[0].bytes(), Some(&[17; PAYLOAD][..]));
    assert_eq!(active(&ports), observed.charged_bytes());
    drop(observed);
    drop(window);
    assert_eq!(active(&ports), 0);
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

fn selected_read(
    window: &PhysicalRecoveryReadAllocation<'_>,
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
) -> Result<Outcome<FundedRecoveryWalObservations>, FundedRecoveryWalReadFailure> {
    window
        .read_wal_source(
            source::WalReadSource::Recovery(discovery),
            segments(4),
            ReadGrant::ceiling_only(),
            Some(&SingleFile),
        )
        .map_err(GrantedReadStop::unread)
}

fn active(ports: &crate::physical_runtime::record_serving::RecordFramePorts) -> u64 {
    ports
        .allocation_events()
        .snapshot()
        .for_dimension(Dimension::OperationScope(Scope::Recovery))
        .active_units()
}

fn pause(role: MediaOperationRole, ordinal: u64) -> (MediaPauseGate, MediaFaultSchedule) {
    let authority =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount)
            .fault_schedule_authority();
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

fn read_at_saturated_boundary(
    coordination: &mut PhysicalRecoveryCoordination,
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    gate: &MediaPauseGate,
    role: MediaOperationRole,
    ordinal: u64,
) -> (
    Result<Outcome<FundedRecoveryWalObservations>, FundedRecoveryWalReadFailure>,
    OperationAllocationGrant,
) {
    let ports = coordination.residency.ports().clone();
    let observer = ports.allocation_events();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let window = coordination.begin_source_read_allocation().unwrap();
            selected_read(&window, discovery)
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while gate.reached_context().is_none() {
            if worker.is_finished() || Instant::now() >= deadline {
                gate.release();
                panic!(
                    "actual selected-read pause not reached: {:?}",
                    worker.join()
                );
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let amount = ORIGINAL
            .checked_sub(active(&ports))
            .and_then(NonZeroU64::new);
        let held = amount.map(|bytes| ports.begin_operation(Scope::Recovery, bytes));
        let before = observer.snapshot().for_dimension(dimension);
        gate.release();
        let result = worker.join().unwrap();
        let context = gate.reached_context().unwrap();
        assert_eq!(context.role(), role);
        assert_eq!(context.role_ordinal(), ordinal);
        let after = observer.snapshot().for_dimension(dimension);
        assert_eq!(
            after.admissions(),
            before.admissions(),
            "known drift or denied payload cannot admit/refund backing"
        );
        assert_eq!(after.admitted_units(), before.admitted_units());
        (
            result,
            held.expect("native competitor headroom required").unwrap(),
        )
    })
}
