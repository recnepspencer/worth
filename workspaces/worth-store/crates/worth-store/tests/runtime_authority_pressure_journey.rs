#![cfg_attr(test, allow(clippy::disallowed_methods))]
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Barrier,
    },
};

use worth_store::physical_runtime::{AdmissionError, PhysicalRuntimeAdmission, PhysicalStore};

#[path = "physical_runtime_journeys/lifecycle_oracle.rs"]
mod lifecycle_oracle;
#[path = "physical_runtime_journeys/pressure_outcome.rs"]
mod pressure_outcome;
#[path = "physical_runtime_journeys/process_death_probe.rs"]
mod process_death_probe;

use lifecycle_oracle::{assert_terminal_observation, LifecycleAction};
use pressure_outcome::PressureOutcome;

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn runtime_authority_pressure_journey_keeps_observation_read_only_and_phase_scoped() {
    if process_death_probe::run_child_mode() {
        return;
    }

    let first = run_pressure_world();
    let replay = run_pressure_world();
    assert_eq!(replay, first);
}

fn run_pressure_world() -> PressureOutcome {
    let counters_before = PhysicalStore::diagnostics();
    prove_concurrent_observers_close_together();
    prove_abort_and_unexpected_drop_make_observers_stale();
    prove_same_root_denial_does_not_block_different_root_admission();
    prove_cancellation_and_panic_release_authority();
    process_death_probe::prove_process_death_emits_no_physical_residue(unique_absent_root(
        "child-death",
    ));
    let outcome = PressureOutcome::between(counters_before, PhysicalStore::diagnostics());
    assert_eq!(outcome, PressureOutcome::EXPECTED);
    outcome
}

fn prove_concurrent_observers_close_together() {
    const OBSERVER_COUNT: usize = 8;

    let root = unique_absent_root("close-pressure");
    let before = snapshot_root(&root);
    let runtime = PhysicalStore::admit(valid_admission(root.clone())).unwrap();
    let runtime_identity = runtime.runtime_identity();
    let observation_handles = (0..OBSERVER_COUNT)
        .map(|_| runtime.observe())
        .collect::<Vec<_>>();
    assert!(observation_handles
        .iter()
        .all(|handle| handle.runtime_identity() == runtime_identity));
    let initial_observation = observation_handles[0].snapshot().unwrap();
    let admitted_generation = initial_observation.lifecycle().generation();
    let admitted_barrier = Arc::new(Barrier::new(OBSERVER_COUNT + 1));
    let closed_barrier = Arc::new(Barrier::new(OBSERVER_COUNT + 1));

    let observers = observation_handles
        .into_iter()
        .map(|handle| {
            let admitted_barrier = Arc::clone(&admitted_barrier);
            let closed_barrier = Arc::clone(&closed_barrier);
            std::thread::spawn(move || {
                let capabilities = handle.installed_capabilities().unwrap();
                assert_all_physical_capabilities_absent(capabilities);
                let admitted = handle.snapshot().unwrap();
                admitted_barrier.wait();
                closed_barrier.wait();
                (admitted, handle.snapshot())
            })
        })
        .collect::<Vec<_>>();

    admitted_barrier.wait();
    let closed = runtime.close();
    closed_barrier.wait();

    for observer in observers {
        let (admitted, terminal) = observer.join().expect("observer must not panic");
        assert_eq!(
            admitted.runtime_identity(),
            initial_observation.runtime_identity()
        );
        assert_eq!(admitted.lifecycle(), initial_observation.lifecycle());
        assert_eq!(admitted.counters().physical_owner_count(), 0);
        assert_eq!(admitted.counters().physical_operation_attempts(), 0);
        assert_eq!(admitted.counters().publication_attempts(), 0);
        assert_eq!(admitted.counters().media_operations(), 0);
        assert_terminal_observation(
            LifecycleAction::Close,
            terminal,
            runtime_identity,
            admitted_generation,
        );
    }

    let readmitted = PhysicalStore::admit(valid_admission(root.clone()))
        .expect("a closed summary must not retain root admission authority");
    assert_ne!(readmitted.runtime_identity(), closed.runtime_identity());
    let readmitted_closed = readmitted.close();
    drop((closed, readmitted_closed));
    assert_eq!(snapshot_root(&root), before);
}

fn prove_abort_and_unexpected_drop_make_observers_stale() {
    let aborted_root = unique_absent_root("abort-pressure");
    let aborted_root_before = snapshot_root(&aborted_root);
    let aborted_runtime = PhysicalStore::admit(valid_admission(aborted_root.clone())).unwrap();
    let aborted_identity = aborted_runtime.runtime_identity();
    let aborted_observer = aborted_runtime.observe();
    let aborted_generation = aborted_observer
        .snapshot()
        .unwrap()
        .lifecycle()
        .generation();
    let aborted = aborted_runtime.abort();
    assert_eq!(aborted.runtime_identity(), aborted_identity);
    assert_eq!(aborted.declared_store_root().as_path(), aborted_root);
    assert_terminal_observation(
        LifecycleAction::Abort,
        aborted_observer.snapshot(),
        aborted_identity,
        aborted_generation,
    );

    let dropped_root = unique_absent_root("drop-pressure");
    let dropped_root_before = snapshot_root(&dropped_root);
    let dropped_runtime = PhysicalStore::admit(valid_admission(dropped_root.clone())).unwrap();
    let dropped_identity = dropped_runtime.runtime_identity();
    let dropped_observer = dropped_runtime.observe();
    let dropped_generation = dropped_observer
        .snapshot()
        .unwrap()
        .lifecycle()
        .generation();
    drop(dropped_runtime);
    assert_terminal_observation(
        LifecycleAction::UnexpectedDrop,
        dropped_observer.snapshot(),
        dropped_identity,
        dropped_generation,
    );

    let readmitted_after_abort =
        PhysicalStore::admit(valid_admission(aborted_root.clone())).unwrap();
    let readmitted_after_drop =
        PhysicalStore::admit(valid_admission(dropped_root.clone())).unwrap();
    drop((
        aborted,
        aborted_observer,
        dropped_observer,
        readmitted_after_abort.abort(),
        readmitted_after_drop.abort(),
    ));
    assert_eq!(snapshot_root(&aborted_root), aborted_root_before);
    assert_eq!(snapshot_root(&dropped_root), dropped_root_before);
}

fn prove_same_root_denial_does_not_block_different_root_admission() {
    const CONTENDER_COUNT: usize = 3;

    let primary_root = unique_absent_root("same-root-pressure");
    let different_root = unique_absent_root("different-root-pressure");
    let primary_root_before = snapshot_root(&primary_root);
    let different_root_before = snapshot_root(&different_root);
    let primary_runtime = PhysicalStore::admit(valid_admission(primary_root.clone())).unwrap();
    let attempt = Arc::new(Barrier::new(CONTENDER_COUNT + 1));
    let release = Arc::new(Barrier::new(CONTENDER_COUNT + 1));
    let contender_roots = [
        ContenderRoot::Same(primary_root.clone()),
        ContenderRoot::Same(primary_root.clone()),
        ContenderRoot::Different(different_root.clone()),
    ];
    let contenders = contender_roots
        .into_iter()
        .map(|contender_root| {
            let attempt = Arc::clone(&attempt);
            let release = Arc::clone(&release);
            std::thread::spawn(move || {
                attempt.wait();
                let outcome = PhysicalStore::admit(valid_admission(contender_root.path()));
                let classification = match (contender_root, &outcome) {
                    (
                        ContenderRoot::Same(_),
                        Err(AdmissionError::DeclaredRootAlreadyAdmitted(_)),
                    ) => AdmissionClassification::SameRootDenied,
                    (ContenderRoot::Different(_), Ok(_)) => {
                        AdmissionClassification::DifferentRootAdmitted
                    }
                    (ContenderRoot::Same(_), Ok(_)) => {
                        panic!("a same-root contender unexpectedly admitted")
                    }
                    (ContenderRoot::Same(_), Err(error)) => {
                        panic!("a same-root contender returned the wrong denial: {error}")
                    }
                    (ContenderRoot::Different(_), Err(error)) => {
                        panic!("the different-root contender was denied: {error}")
                    }
                };
                release.wait();
                if let Ok(runtime) = outcome {
                    runtime.close();
                }
                classification
            })
        })
        .collect::<Vec<_>>();

    attempt.wait();
    release.wait();
    let classifications = contenders
        .into_iter()
        .map(|contender| contender.join().expect("contender must not panic"))
        .collect::<Vec<_>>();
    assert_eq!(
        classifications
            .iter()
            .filter(|classification| **classification == AdmissionClassification::SameRootDenied)
            .count(),
        2
    );
    assert_eq!(
        classifications
            .iter()
            .filter(|classification| {
                **classification == AdmissionClassification::DifferentRootAdmitted
            })
            .count(),
        1
    );

    primary_runtime.close();
    PhysicalStore::admit(valid_admission(primary_root.clone()))
        .expect("the primary root authority must be released after close")
        .close();
    PhysicalStore::admit(valid_admission(different_root.clone()))
        .expect("the different-root contender must release its authority")
        .close();
    assert_eq!(snapshot_root(&primary_root), primary_root_before);
    assert_eq!(snapshot_root(&different_root), different_root_before);
}

fn prove_cancellation_and_panic_release_authority() {
    let cancelled_root = unique_absent_root("cancel-pressure");
    let cancelled_before = snapshot_root(&cancelled_root);
    let cancelled = valid_admission(cancelled_root.clone()).cancel();
    assert_eq!(cancelled.declared_store_root().as_path(), cancelled_root);
    PhysicalStore::admit(valid_admission(cancelled_root.clone()))
        .expect("cancelling a request must not reserve root authority")
        .close();

    let panic_root = unique_absent_root("panic-pressure");
    let panic_before = snapshot_root(&panic_root);
    let runtime = PhysicalStore::admit(valid_admission(panic_root.clone())).unwrap();
    let runtime_identity = runtime.runtime_identity();
    let observer = runtime.observe();
    let admitted_generation = observer.snapshot().unwrap().lifecycle().generation();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _runtime_owner = runtime;
        panic!("controlled live-runtime panic");
    }));
    assert!(panic.is_err());
    assert_terminal_observation(
        LifecycleAction::Panic,
        observer.snapshot(),
        runtime_identity,
        admitted_generation,
    );
    PhysicalStore::admit(valid_admission(panic_root.clone()))
        .expect("panic unwinding must release root authority")
        .close();

    assert_eq!(snapshot_root(&cancelled_root), cancelled_before);
    assert_eq!(snapshot_root(&panic_root), panic_before);
}

fn assert_all_physical_capabilities_absent(
    capabilities: worth_store::physical_runtime::InstalledCapabilityStatus,
) {
    use worth_store::physical_runtime::{CapabilityAvailability, PhysicalCapability};

    for capability in [
        PhysicalCapability::Media,
        PhysicalCapability::PageRecord,
        PhysicalCapability::WalCheckpoint,
        PhysicalCapability::Recovery,
        PhysicalCapability::Maintenance,
        PhysicalCapability::Layout,
        PhysicalCapability::Blob,
    ] {
        assert_eq!(
            capabilities.availability(capability),
            CapabilityAvailability::Absent
        );
    }
}

enum ContenderRoot {
    Same(PathBuf),
    Different(PathBuf),
}

impl ContenderRoot {
    fn path(&self) -> PathBuf {
        match self {
            Self::Same(path) | Self::Different(path) => path.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdmissionClassification {
    SameRootDenied,
    DifferentRootAdmitted,
}

#[derive(Debug, PartialEq, Eq)]
enum RootSnapshot {
    Absent,
    Directory(Vec<OsString>),
}

fn valid_admission(root: PathBuf) -> PhysicalRuntimeAdmission {
    PhysicalRuntimeAdmission::new(root).expect("the fixed root declaration should validate")
}

fn unique_absent_root(label: &str) -> PathBuf {
    let sequence = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "worth-store-c3-{label}-{}-{sequence}",
        std::process::id(),
    ))
}

fn snapshot_root(root: &Path) -> RootSnapshot {
    if !root.exists() {
        return RootSnapshot::Absent;
    }

    let mut entries = std::fs::read_dir(root)
        .expect("the root should remain inspectable through ordinary OS APIs")
        .map(|entry| entry.expect("directory entry must be readable").file_name())
        .collect::<Vec<_>>();
    entries.sort();
    RootSnapshot::Directory(entries)
}
