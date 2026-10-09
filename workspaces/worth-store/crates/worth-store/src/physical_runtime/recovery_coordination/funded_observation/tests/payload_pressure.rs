//! Competition at the real metadata boundary isolates payload allocation from
//! the earlier address/open admissions, which have already settled.

use super::*;
use crate::physical_runtime::certification::{
    MediaFaultDirective, MediaOperationRole, MediaPauseGate,
};
use std::time::{Duration, Instant};

#[test]
fn native_file_open_pressure_denies_after_address_before_payload_and_retries() {
    use worth_store_physical_backend::ArtifactTreeDirectory;

    let (directory, media, mut coordination) = world();
    write_checkpoint(&directory, &[41; 64]);
    let ports = coordination.residency.ports().clone();
    let original = coordination
        .recovery_allocation_admission()
        .unwrap()
        .byte_limit();
    // Independently census the actual checkpoint address. Reserving exactly
    // that storage allows its construction but leaves no room for cap open.
    let file = ArtifactTreeDirectory::families()
        .file("checkpoint.current")
        .unwrap();
    let address_bytes = file.owned_heap_bytes().unwrap();
    let held = ports
        .begin_operation(
            Scope::Recovery,
            NonZeroU64::new(original - address_bytes).unwrap(),
        )
        .unwrap();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let failure = coordination
        .begin_source_read_allocation()
        .unwrap()
        .read_checkpoint(&mut discovery, ReadGrant::ceiling_only())
        .observed()
        .unwrap_err();
    assert!(
        matches!(failure, RecoveryDiscoveryAllocationFailure::Allocation {
        artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
        requested,
        cause: PhysicalRecoveryObservationAllocationDenial::PathResidency {
            boundary: ArtifactTreePathAllocationBoundary::FileOpen,
            cause: PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
        }, ..
    } if requested > 0 && required == original + requested as u64 && admitted == original)
    );
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        held.bytes()
    );
    drop(held);
    let observed = coordination
        .begin_source_read_allocation()
        .unwrap()
        .read_checkpoint(&mut discovery, ReadGrant::ceiling_only())
        .observed()
        .unwrap();
    assert_eq!(observed.observed().bytes(), Some(&[41; 64][..]));
    drop(observed);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

#[test]
fn metadata_boundary_pressure_denies_payload_then_same_owner_retries() {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            MediaOperationRole::ReadMetadata,
            2, // identity qualification precedes the actual checkpoint length
            MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    let (directory, media, mut coordination) = world_with_schedule(Some(schedule));
    write_checkpoint(&directory, &[41; 64]);
    let ports = coordination.residency.ports().clone();
    let original = coordination
        .recovery_allocation_admission()
        .unwrap()
        .byte_limit();
    let mut discovery = media.bounded_discovery(2, 4096).unwrap();
    let (result, held) = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            coordination
                .begin_source_read_allocation()
                .unwrap()
                .read_checkpoint(&mut discovery, ReadGrant::ceiling_only())
                .observed()
        });
        wait_for_metadata(&gate, &worker);
        let active = ports.counters().active_operation_bytes_for(Scope::Recovery);
        let held = ports.begin_operation(
            Scope::Recovery,
            NonZeroU64::new(original - active - 63).unwrap(),
        );
        gate.release();
        (worker.join().unwrap(), held.unwrap())
    });
    assert_eq!(
        gate.reached_context().unwrap().role(),
        MediaOperationRole::ReadMetadata
    );
    let failure = result.unwrap_err();
    assert!(
        matches!(failure, RecoveryDiscoveryAllocationFailure::Allocation {
        artifact: RecoveryDiscoveryArtifact::CurrentCheckpoint,
        requested: 64,
        cause: PhysicalRecoveryObservationAllocationDenial::Residency(
            PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }
        ), ..
    } if required == original + 1 && admitted == original)
    );
    assert_eq!(discovery.counters().bytes_read, 0);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        held.bytes()
    );
    assert_eq!(
        std::fs::read(directory.path().join("store/families/checkpoint.current")).unwrap(),
        [41; 64]
    );
    drop(held);
    let observation = coordination
        .begin_source_read_allocation()
        .unwrap()
        .read_checkpoint(&mut discovery, ReadGrant::ceiling_only())
        .observed()
        .unwrap();
    assert_eq!(observation.observed().bytes(), Some(&[41; 64][..]));
    assert_eq!(observation.charged_bytes(), 64);
    drop(coordination);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        64
    );
    drop(observation);
    assert_eq!(
        ports.counters().active_operation_bytes_for(Scope::Recovery),
        0
    );
    assert_eq!(discovery.finish().recovery_effect_count(), 0);
}

fn wait_for_metadata<T>(gate: &MediaPauseGate, worker: &std::thread::ScopedJoinHandle<'_, T>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while gate.reached_context().is_none() {
        if worker.is_finished() || Instant::now() >= deadline {
            gate.release();
            panic!("the actual metadata pause was not reached");
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
