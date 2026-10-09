//! Native read denial and escaped diagnostic ownership on genuine WAL input.

use super::*;
use std::time::{Duration, Instant};
use worth_store::physical_runtime::{
    certification::{MediaFaultDirective, MediaFaultSchedule, MediaOperationRole, MediaPauseGate},
    BoundedRecoveryFilesystemDiscovery, FilesystemAccessPosture, FilesystemMediaAdmission,
    FundedRecoveryWalReadFailure,
};

pub(super) fn payload_pause() -> (MediaPauseGate, MediaFaultSchedule) {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let schedule = authority
        .schedule(vec![authority.rule(
            // Identity, checkpoint and source-root metadata precede WAL.
            MediaOperationRole::ReadMetadata,
            4,
            MediaFaultDirective::PauseAfter(gate.clone()),
        )])
        .unwrap();
    (gate, schedule)
}

pub(super) fn deny_before_discovery(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
) -> u64 {
    let observer = coordination.owner().certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let prior = observer.snapshot().for_dimension(dimension).active_units();
    let held = coordination
        .owner()
        .certification_begin_recovery_allocation(NonZeroU64::new(ORIGINAL - prior - 1).unwrap())
        .unwrap();
    let before = observer.snapshot().for_dimension(dimension);
    let read_before = discovery.counters().wal_bytes_read;
    let entries_before = discovery.counters().directory_entries_observed;
    assert_eq!(before.active_units(), ORIGINAL - 1);
    let failure = match observe_wal(discovery, coordination, limits, counters) {
        Err(failure) => failure,
        Ok(_) => panic!("WAL diagnostic backing must deny before discovery"),
    };
    let [PhysicalRecoverySourceDenial::WalRead {
        failure: read_failure,
    }] = failure.source_denials.as_slice()
    else {
        panic!(
            "exact shared WAL read cause required: {:?}",
            failure.source_denials
        );
    };
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalDirectory,
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::Residency(
                PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            ),
        ..
    } = read_failure.diagnostic()
    else {
        panic!("diagnostic preparation must be the responsible boundary");
    };
    assert!(requested > 1);
    let diagnostic_bytes = requested as u64;
    assert_eq!(*required, ORIGINAL - 1 + diagnostic_bytes);
    assert_eq!(*admitted, ORIGINAL);
    assert_eq!(read_failure.charged_bytes(), 0);
    assert_eq!(discovery.counters().wal_bytes_read, read_before);
    assert_eq!(
        discovery.counters().directory_entries_observed,
        entries_before
    );
    assert!(failure.integrity_observations.wal().is_empty());
    let denied = observer.snapshot().for_dimension(dimension);
    assert_eq!(denied.admissions(), before.admissions());
    assert_eq!(denied.admitted_units(), before.admitted_units());
    assert_eq!(denied.denials(), before.denials() + 1);
    assert_eq!(denied.active_units(), before.active_units());
    let clone_before = observer.snapshot();
    let inline_clone = read_failure.clone();
    assert_eq!(inline_clone, *read_failure);
    assert_eq!(observer.snapshot(), clone_before);
    drop(failure);
    assert_eq!(inline_clone.charged_bytes(), 0);
    drop(inline_clone);
    drop(held);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        prior
    );
    diagnostic_bytes
}

pub(super) fn deny_payload_and_retain_context(
    root: &Path,
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    coordination: &mut RecoveryCoordination,
    limits: PhysicalRecoveryLimits,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    diagnostic_bytes: u64,
    gate: &MediaPauseGate,
) -> FundedRecoveryWalReadFailure {
    let observer = coordination.owner().certification_residency_allocations();
    let dimension = Dimension::OperationScope(Scope::Recovery);
    let prior = observer.snapshot().for_dimension(dimension).active_units();
    let directory = root.join("families/wal");
    let mut entries = std::fs::read_dir(&directory).unwrap();
    let name_bytes = entries
        .next()
        .unwrap()
        .unwrap()
        .file_name()
        .as_encoded_bytes()
        .len() as u64;
    for entry in entries {
        assert_eq!(
            entry.unwrap().file_name().as_encoded_bytes().len() as u64,
            name_bytes
        );
    }
    let mut held = coordination
        .owner()
        .certification_begin_recovery_allocation(NonZeroU64::MIN)
        .unwrap();
    let read_before = discovery.counters().wal_bytes_read;
    let result = std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .stack_size(16 << 20)
            .spawn_scoped(scope, || {
                observe_wal(discovery, coordination, limits, counters)
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while gate.reached_context().is_none() {
            if worker.is_finished() || Instant::now() >= deadline {
                gate.release();
                match worker.join().unwrap() {
                    Err(failure) => panic!("metadata setup failed: {:?}", failure.source_denials),
                    Ok(_) => panic!("actual metadata completed without the expected pause"),
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let active = observer.snapshot().for_dimension(dimension).active_units();
        let target = ORIGINAL
            .checked_sub(active)
            .and_then(|bytes| bytes.checked_add(held.bytes()));
        let growth = target.map(|bytes| held.try_resize(bytes));
        gate.release();
        let result = worker.join().unwrap();
        let context = gate.reached_context().unwrap();
        assert_eq!(context.role(), MediaOperationRole::ReadMetadata);
        assert_eq!(context.role_ordinal(), 4);
        growth.expect("fixture has real residual headroom").unwrap();
        result
    });
    let failure = match result {
        Err(failure) => failure,
        Ok(_) => panic!("known WAL payload must deny after its name is funded"),
    };
    let [PhysicalRecoverySourceDenial::WalRead {
        failure: read_failure,
    }] = failure.source_denials.as_slice()
    else {
        panic!(
            "named shared read failure required: {:?}",
            failure.source_denials
        );
    };
    let RecoveryWalReadFailureView::Allocation {
        artifact: RecoveryWalArtifactView::WalArtifact(name),
        requested,
        cause:
            PhysicalRecoveryObservationAllocationDenial::Residency(
                PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted },
            ),
        ..
    } = read_failure.diagnostic()
    else {
        panic!("payload denial must retain the genuine source name");
    };
    assert_eq!(
        requested as u64,
        std::fs::metadata(directory.join(name)).unwrap().len()
    );
    assert_eq!(*required, ORIGINAL + requested as u64);
    assert_eq!(*admitted, ORIGINAL);
    assert_eq!(discovery.counters().wal_bytes_read, read_before);
    assert_eq!(read_failure.charged_bytes(), diagnostic_bytes + name_bytes);
    let before_clone = observer.snapshot();
    let shared_denial = failure.source_denials[0].clone();
    assert_eq!(
        observer.snapshot(),
        before_clone,
        "sharing admits no new storage"
    );
    assert_eq!(shared_denial, failure.source_denials[0]);
    drop(failure);
    let PhysicalRecoverySourceDenial::WalRead { failure: shared } = shared_denial else {
        panic!("cloning must preserve the funded outer diagnostic");
    };
    drop(held);
    assert_eq!(
        observer.snapshot().for_dimension(dimension).active_units(),
        prior + shared.charged_bytes(),
        "read roster unwinds but diagnostic remains funded"
    );
    shared
}
