pub(crate) use {
    std::alloc::System, std::ffi::OsString, std::path::Path, std::path::PathBuf,
    std::sync::atomic::AtomicU64, std::sync::atomic::Ordering,
};

pub(crate) use {
    stats_alloc::Region, stats_alloc::Stats, stats_alloc::StatsAlloc,
    stats_alloc::INSTRUMENTED_SYSTEM,
};
pub(crate) use {
    worth_store::physical_runtime::AdmissionError,
    worth_store::physical_runtime::CapabilityAvailability,
    worth_store::physical_runtime::DeclaredStoreRootDenialKind,
    worth_store::physical_runtime::LifecycleObservation,
    worth_store::physical_runtime::PhysicalCapability,
    worth_store::physical_runtime::PhysicalRuntimeAdmission,
    worth_store::physical_runtime::PhysicalStore,
    worth_store::physical_runtime::ProcessRuntimeCounterSnapshot,
    worth_store::physical_runtime::RootAdmissionObservation,
};

pub(crate) use {
    crate::lifecycle_oracle::assert_terminal_observation, crate::lifecycle_oracle::LifecycleAction,
};

pub(crate) static NEXT_TEST_ROOT: AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[global_allocator]
pub(crate) static GLOBAL_ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

pub(crate) const MAX_ADMISSION_ALLOCATION_BYTES: usize = 2_048;

pub(crate) fn prove_invalid_admission_is_allocation_free() {
    let invalid_root = std::path::PathBuf::from("relative-store-root");
    let allocation_region = stats_alloc::Region::new(GLOBAL_ALLOCATOR);
    let outcome = worth_store::physical_runtime::PhysicalRuntimeAdmission::new(invalid_root);
    let allocation = allocation_region.change();

    assert!(matches!(
        outcome,
        Err(
            worth_store::physical_runtime::AdmissionError::InvalidDeclaredStoreRoot {
                kind: worth_store::physical_runtime::DeclaredStoreRootDenialKind::Relative,
                ..
            }
        )
    ));
    assert_eq!(
        allocation,
        stats_alloc::Stats::default(),
        "invalid declaration must be rejected before registration or allocation"
    );
}

pub(crate) fn prove_declared_root_closes_without_retaining_authority() {
    let declared_root = unique_absent_root();
    let initial_absent_root = snapshot_root(&declared_root);
    assert_eq!(
        initial_absent_root,
        crate::sealed_runtime_lifecycle_journey::RootSnapshot::Absent
    );

    let first_admission = valid_admission(declared_root.clone());
    let second_admission = valid_admission(declared_root.clone());

    let first_allocation_region = stats_alloc::Region::new(GLOBAL_ALLOCATOR);
    let first_runtime = worth_store::physical_runtime::PhysicalStore::admit(first_admission)
        .expect("the validated root declaration should admit");
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

    let observation_allocation_region = stats_alloc::Region::new(GLOBAL_ALLOCATOR);
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
        stats_alloc::Stats::default(),
        "identity and lifecycle observation must be allocation-free after handle acquisition"
    );
    assert_eq!(facade_capabilities, observed_capabilities);
    for capability in [
        worth_store::physical_runtime::PhysicalCapability::Media,
        worth_store::physical_runtime::PhysicalCapability::PageRecord,
        worth_store::physical_runtime::PhysicalCapability::WalCheckpoint,
        worth_store::physical_runtime::PhysicalCapability::Recovery,
        worth_store::physical_runtime::PhysicalCapability::Maintenance,
        worth_store::physical_runtime::PhysicalCapability::Layout,
        worth_store::physical_runtime::PhysicalCapability::Blob,
    ] {
        assert_eq!(
            facade_capabilities.availability(capability),
            worth_store::physical_runtime::CapabilityAvailability::Absent
        );
    }
    assert_eq!(
        first_snapshot.runtime_identity(),
        second_snapshot.runtime_identity()
    );
    assert_eq!(first_snapshot.lifecycle(), second_snapshot.lifecycle());
    assert_eq!(
        first_snapshot.root_admission(),
        worth_store::physical_runtime::RootAdmissionObservation::Admitted
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
        worth_store::physical_runtime::LifecycleObservation::Admitted { .. }
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
        crate::lifecycle_oracle::LifecycleAction::Close,
        first_observer.snapshot(),
        first_identity,
        admitted_generation,
    );
    assert_terminal_observation(
        crate::lifecycle_oracle::LifecycleAction::Close,
        second_observer.snapshot(),
        first_identity,
        admitted_generation,
    );

    let second_allocation_region = stats_alloc::Region::new(GLOBAL_ALLOCATOR);
    let second_runtime = worth_store::physical_runtime::PhysicalStore::admit(second_admission)
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

pub(crate) fn prove_existing_root_is_unchanged() {
    let existing_empty_root = unique_absent_root();
    std::fs::create_dir(&existing_empty_root)
        .expect("the existing-root world should create one empty directory");
    let initial_empty_root = snapshot_root(&existing_empty_root);
    let existing_runtime = worth_store::physical_runtime::PhysicalStore::admit(valid_admission(
        existing_empty_root.clone(),
    ))
    .expect("an existing empty directory remains only a declaration in C.3");
    let existing_identity = existing_runtime.runtime_identity();
    let existing_closed = existing_runtime.close();
    assert_eq!(existing_closed.runtime_identity(), existing_identity);
    assert_eq!(snapshot_root(&existing_empty_root), initial_empty_root);
    std::fs::remove_dir(&existing_empty_root)
        .expect("the unchanged empty test directory should be removable");
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RootSnapshot {
    Absent,
    Directory(Vec<OsString>),
}

pub(crate) fn valid_admission(root: PathBuf) -> PhysicalRuntimeAdmission {
    worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root)
        .expect("the fixed root declaration should validate")
}

pub(crate) fn unique_absent_root() -> PathBuf {
    let sequence = NEXT_TEST_ROOT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "worth-store-c3-admission-{}-{sequence}",
        std::process::id(),
    ))
}

pub(crate) fn snapshot_root(root: &Path) -> RootSnapshot {
    if !root.exists() {
        return crate::sealed_runtime_lifecycle_journey::RootSnapshot::Absent;
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
    crate::sealed_runtime_lifecycle_journey::RootSnapshot::Directory(entries)
}

pub(crate) fn admission_allocation_footprint(stats: Stats) -> (usize, usize, usize, isize) {
    (
        stats.allocations,
        stats.bytes_allocated,
        stats.reallocations,
        stats.bytes_reallocated,
    )
}

pub(crate) fn assert_process_counter_delta(
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
