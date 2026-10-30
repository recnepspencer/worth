#![cfg_attr(test, allow(clippy::disallowed_methods))]
use std::{
    alloc::System,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use stats_alloc::{Region, Stats, StatsAlloc, INSTRUMENTED_SYSTEM};
use worth_store::physical_runtime::{
    AdmissionError, CapabilityAvailability, DeclaredStoreRootDenialKind, LifecycleObservation,
    PhysicalCapability, PhysicalRuntimeAdmission, PhysicalStore, ProcessRuntimeCounterSnapshot,
    RootAdmissionObservation,
};

#[path = "physical_runtime_journeys/lifecycle_oracle.rs"]
mod lifecycle_oracle;

use lifecycle_oracle::{assert_terminal_observation, LifecycleAction};

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

#[global_allocator]
static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

const MAX_ADMISSION_ALLOCATION_BYTES: usize = 2_048;

#[test]
fn sealed_runtime_lifecycle_journey_establishes_the_admission_foundation() {
    let counters_before = PhysicalStore::diagnostics();
    prove_invalid_admission_is_allocation_free();
    prove_declared_root_closes_without_retaining_authority();
    prove_existing_root_is_unchanged();
    assert_process_counter_delta(counters_before, PhysicalStore::diagnostics());
}

fn prove_invalid_admission_is_allocation_free() {
    let invalid_root = PathBuf::from("relative-store-root");
    let allocation_region = Region::new(GLOBAL_ALLOCATOR);
    let outcome = PhysicalRuntimeAdmission::new(invalid_root);
    let allocation = allocation_region.change();

    assert!(matches!(
        outcome,
        Err(AdmissionError::InvalidDeclaredStoreRoot {
            kind: DeclaredStoreRootDenialKind::Relative,
            ..
        })
    ));
    assert_eq!(
        allocation,
        Stats::default(),
        "invalid declaration must be rejected before registration or allocation"
    );
}

fn prove_declared_root_closes_without_retaining_authority() {
    let declared_root = unique_absent_root();
    let initial_absent_root = snapshot_root(&declared_root);
    assert_eq!(initial_absent_root, RootSnapshot::Absent);

    let first_admission = valid_admission(declared_root.clone());
    let second_admission = valid_admission(declared_root.clone());

    let first_allocation_region = Region::new(GLOBAL_ALLOCATOR);
    let first_runtime =
        PhysicalStore::admit(first_admission).expect("the validated root declaration should admit");
    let first_allocation = first_allocation_region.change();
    assert!(
        first_allocation.bytes_allocated <= MAX_ADMISSION_ALLOCATION_BYTES,
        "admission allocated {} bytes, exceeding the {}-byte lifecycle-owner ceiling",
        first_allocation.bytes_allocated,
        MAX_ADMISSION_ALLOCATION_BYTES
    );
    assert_eq!(first_runtime.declared_store_root().as_path(), declared_root);

    let first_observer = first_runtime.observe();
    let second_observer = first_runtime.observe();
    assert_eq!(
        first_observer.runtime_identity(),
        first_runtime.runtime_identity()
    );
    assert_eq!(
        second_observer.runtime_identity(),
        first_runtime.runtime_identity()
    );

    let observation_allocation_region = Region::new(GLOBAL_ALLOCATOR);
    let facade_capabilities = first_runtime.installed_capabilities();
    let observed_capabilities = first_observer
        .installed_capabilities()
        .expect("capability status should remain observable while admitted");
    let first_snapshot = first_observer
        .snapshot()
        .expect("the admitted runtime should be observable");
    let second_snapshot = second_observer
        .snapshot()
        .expect("simultaneous admitted observations should remain valid");
    let observation_allocation = observation_allocation_region.change();
    assert_eq!(
        observation_allocation,
        Stats::default(),
        "identity and lifecycle observation must be allocation-free after handle acquisition"
    );
    assert_eq!(facade_capabilities, observed_capabilities);
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
            facade_capabilities.availability(capability),
            CapabilityAvailability::Absent
        );
    }
    assert_eq!(
        first_snapshot.runtime_identity(),
        second_snapshot.runtime_identity()
    );
    assert_eq!(first_snapshot.lifecycle(), second_snapshot.lifecycle());
    assert_eq!(
        first_snapshot.root_admission(),
        RootAdmissionObservation::Admitted
    );
    assert_eq!(
        first_snapshot.root_admission(),
        second_snapshot.root_admission()
    );
    assert_eq!(
        first_snapshot.runtime_identity(),
        first_runtime.runtime_identity()
    );
    assert!(matches!(
        first_snapshot.lifecycle(),
        LifecycleObservation::Admitted { .. }
    ));
    assert_eq!(first_snapshot.counters().physical_owner_count(), 0);
    assert_eq!(first_snapshot.counters().physical_operation_attempts(), 0);
    assert_eq!(first_snapshot.counters().publication_attempts(), 0);
    assert_eq!(first_snapshot.counters().media_operations(), 0);
    let admitted_generation = first_snapshot.lifecycle().generation();

    let first_identity = first_runtime.runtime_identity();
    let closed = first_runtime.close();
    assert_eq!(closed.runtime_identity(), first_identity);
    assert_eq!(closed.declared_store_root().as_path(), declared_root);
    let closed_counters = closed.counters();
    assert_eq!(closed_counters.admission_attempts(), 1);
    assert_eq!(closed_counters.admitted_incarnations(), 1);
    assert_eq!(closed_counters.observation_acquisitions(), 2);
    assert_eq!(closed_counters.active_observations(), 2);
    assert_eq!(closed_counters.lifecycle_observations(), 2);
    assert_eq!(closed_counters.capability_observations(), 14);
    assert_eq!(closed_counters.explicit_closes(), 1);
    assert_eq!(closed_counters.explicit_aborts(), 0);
    assert_eq!(closed_counters.panic_terminations(), 0);
    assert_eq!(closed_counters.unexpected_drops(), 0);
    assert_terminal_observation(
        LifecycleAction::Close,
        first_observer.snapshot(),
        first_identity,
        admitted_generation,
    );
    assert_terminal_observation(
        LifecycleAction::Close,
        second_observer.snapshot(),
        first_identity,
        admitted_generation,
    );

    let second_allocation_region = Region::new(GLOBAL_ALLOCATOR);
    let second_runtime = PhysicalStore::admit(second_admission)
        .expect("dropping the composition owner should release its declaration");
    let second_allocation = second_allocation_region.change();
    assert_eq!(
        admission_allocation_footprint(second_allocation),
        admission_allocation_footprint(first_allocation),
        "the last owner must release registry storage instead of retaining its high-water allocation"
    );
    assert_ne!(second_runtime.runtime_identity(), first_identity);
    let second_identity = second_runtime.runtime_identity();
    let second_closed = second_runtime.close();
    assert_eq!(second_closed.runtime_identity(), second_identity);

    drop((closed, second_closed, first_observer, second_observer));

    assert_eq!(snapshot_root(&declared_root), initial_absent_root);
}

fn prove_existing_root_is_unchanged() {
    let existing_empty_root = unique_absent_root();
    std::fs::create_dir(&existing_empty_root)
        .expect("the existing-root world should create one empty directory");
    let initial_empty_root = snapshot_root(&existing_empty_root);
    let existing_runtime = PhysicalStore::admit(valid_admission(existing_empty_root.clone()))
        .expect("an existing empty directory remains only a declaration in C.3");
    let existing_identity = existing_runtime.runtime_identity();
    let existing_closed = existing_runtime.close();
    assert_eq!(existing_closed.runtime_identity(), existing_identity);
    assert_eq!(snapshot_root(&existing_empty_root), initial_empty_root);
    std::fs::remove_dir(&existing_empty_root)
        .expect("the unchanged empty test directory should be removable");
}

#[derive(Debug, PartialEq, Eq)]
enum RootSnapshot {
    Absent,
    Directory(Vec<OsString>),
}

fn valid_admission(root: PathBuf) -> PhysicalRuntimeAdmission {
    PhysicalRuntimeAdmission::new(root).expect("the fixed root declaration should validate")
}

fn unique_absent_root() -> PathBuf {
    let sequence = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "worth-store-c3-admission-{}-{sequence}",
        std::process::id(),
    ))
}

fn snapshot_root(root: &Path) -> RootSnapshot {
    if !root.exists() {
        return RootSnapshot::Absent;
    }

    let mut entries = std::fs::read_dir(root)
        .expect("the test root should remain inspectable through ordinary OS APIs")
        .map(|entry| {
            entry
                .expect("every directory entry should remain readable")
                .file_name()
        })
        .collect::<Vec<_>>();
    entries.sort();
    RootSnapshot::Directory(entries)
}

fn admission_allocation_footprint(stats: Stats) -> (usize, usize, usize, isize) {
    (
        stats.allocations,
        stats.bytes_allocated,
        stats.reallocations,
        stats.bytes_reallocated,
    )
}

fn assert_process_counter_delta(
    before: ProcessRuntimeCounterSnapshot,
    after: ProcessRuntimeCounterSnapshot,
) {
    assert_eq!(after.admission_attempts(), before.admission_attempts() + 4);
    assert_eq!(
        after.admitted_incarnations(),
        before.admitted_incarnations() + 3
    );
    assert_eq!(after.admission_denials(), before.admission_denials() + 1);
    assert_eq!(
        after.admission_cancellations(),
        before.admission_cancellations()
    );
    assert_eq!(
        after.admission_panics_before_return(),
        before.admission_panics_before_return()
    );
    assert_eq!(
        after.observation_acquisitions(),
        before.observation_acquisitions() + 2
    );
    assert_eq!(after.active_observations(), before.active_observations());
    assert_eq!(
        after.lifecycle_observations(),
        before.lifecycle_observations() + 4
    );
    assert_eq!(
        after.capability_observations(),
        before.capability_observations() + 14
    );
    assert_eq!(after.explicit_closes(), before.explicit_closes() + 3);
    assert_eq!(after.explicit_aborts(), before.explicit_aborts());
    assert_eq!(after.panic_terminations(), before.panic_terminations());
    assert_eq!(after.unexpected_drops(), before.unexpected_drops());
    assert_eq!(after.physical_owner_count(), 0);
    assert_eq!(after.physical_operation_attempts(), 0);
    assert_eq!(after.publication_attempts(), 0);
    assert_eq!(after.media_operations(), 0);
}
