pub(crate) use {std::num::NonZeroU32, std::num::NonZeroU64};

pub(crate) use {worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPressureEvidence, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_format::store_namespace::StableStoreIdentity};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) const PAGE_BYTES: u64 = 16_384;
pub(crate) const SUCCESSOR_SCOPES: [Scope; 5] = [
    worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
    worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
    worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
    worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
    worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
];

pub(crate) fn successor_scope_runtime(root: &std::path::Path) -> ServingPhysicalRuntime {
    let (format, placement, access) = configuration();
    success(initialize_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(format, placement, access, durability)
            .with_residency_policy(successor_policy(format))
    },))
}

pub(crate) fn assert_exact_scope_ceilings(serving: &ServingPhysicalRuntime) {
    let allocations = serving.physical_allocations();
    let store = serving.store_identity();
    let generation = serving.residency_observation().store_generation();
    assert_exact_scope_ceiling!(
        serving,
        allocations,
        store,
        generation,
        admit_recovery,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery
    );
    assert_exact_scope_ceiling!(
        serving,
        allocations,
        store,
        generation,
        admit_scrub,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub
    );
    assert_exact_scope_ceiling!(
        serving,
        allocations,
        store,
        generation,
        admit_maintenance,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance
    );
    assert_exact_scope_ceiling!(
        serving,
        allocations,
        store,
        generation,
        admit_verification,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification
    );
    assert_exact_scope_ceiling!(
        serving,
        allocations,
        store,
        generation,
        admit_blob,
        worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob
    );
}

pub(crate) fn assert_scope_isolation(serving: &ServingPhysicalRuntime) {
    let allocations = serving.physical_allocations();
    let recovery = allocations.admit_recovery(bytes(PAGE_BYTES)).unwrap();
    let scrub = allocations
        .admit_scrub(bytes(PAGE_BYTES))
        .expect("Recovery allocation must not consume Scrub's ceiling");
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery),
        PAGE_BYTES,
    );
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub),
        PAGE_BYTES,
    );
    drop((recovery, scrub));
}

pub(crate) fn assert_global_envelope_and_release(serving: &ServingPhysicalRuntime) {
    let allocations = serving.physical_allocations();
    let recovery = allocations.admit_recovery(bytes(PAGE_BYTES)).unwrap();
    let scrub = allocations.admit_scrub(bytes(PAGE_BYTES)).unwrap();
    let maintenance = allocations.admit_maintenance(bytes(PAGE_BYTES)).unwrap();
    let verification = allocations.admit_verification(bytes(PAGE_BYTES)).unwrap();
    let blob = allocations.admit_blob(bytes(PAGE_BYTES)).unwrap();
    let counters = serving.residency_observation().counters();
    assert_eq!(
        counters.active_operation_bytes(),
        PAGE_BYTES * SUCCESSOR_SCOPES.len() as u64,
    );
    for scope in SUCCESSOR_SCOPES {
        assert_eq!(counters.active_operation_bytes_for(scope), PAGE_BYTES);
    }

    let pressure = allocations
        .admit_recovery(bytes(PAGE_BYTES))
        .expect_err("the combined successor scopes must stop at the global envelope")
        .pressure()
        .expect("global exhaustion must lower to Store pressure evidence");
    assert_pressure(
        pressure,
        ExpectedScopePressure {
            store: serving.store_identity(),
            dimension: worth_store::physical_runtime::PhysicalResidencyDimension::OperationBytes,
            scope: worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
            requested: PAGE_BYTES,
            admitted: PAGE_BYTES * SUCCESSOR_SCOPES.len() as u64,
            limit: PAGE_BYTES * SUCCESSOR_SCOPES.len() as u64,
        },
    );

    drop((recovery, scrub, maintenance, verification, blob));
    assert_released_scope_counters(serving);
}

pub(crate) fn assert_released_scope_counters(serving: &ServingPhysicalRuntime) {
    let released = serving.residency_observation().counters();
    assert_eq!(released.active_operation_bytes(), 0);
    assert_eq!(
        released.peak_operation_bytes(),
        PAGE_BYTES * SUCCESSOR_SCOPES.len() as u64,
    );
    for scope in SUCCESSOR_SCOPES {
        assert_eq!(released.active_operation_bytes_for(scope), 0);
        assert_eq!(released.peak_operation_bytes_for(scope), PAGE_BYTES * 2);
    }
}

pub(crate) fn successor_policy(
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    let operation = bytes(PAGE_BYTES * SUCCESSOR_SCOPES.len() as u64);
    let mut builder = worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(512 * 1024))
        .resident_bytes(bytes(PAGE_BYTES))
        .metadata_bytes(bytes(256 * 1024))
        .frame_entries(frames(8))
        .pinned_frames(frames(8))
        .pin_leases(frames(8))
        .dirty_frames(frames(4))
        .dirty_replacement_bytes(bytes(PAGE_BYTES))
        .operation_bytes(operation)
        .scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead, bytes(PAGE_BYTES))
        .scope_bytes(worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite, bytes(PAGE_BYTES));
    for scope in SUCCESSOR_SCOPES {
        builder = builder.scope_bytes(scope, bytes(PAGE_BYTES * 2));
    }
    builder
        .speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch, frames(8))
        .speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead, frames(8))
        .speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind, frames(4))
        .admit(format)
        .into_result()
        .unwrap()
}

pub(crate) struct ExpectedScopePressure {
    pub(crate) store: StableStoreIdentity,
    pub(crate) dimension: PhysicalResidencyDimension,
    pub(crate) scope: Scope,
    pub(crate) requested: u64,
    pub(crate) admitted: u64,
    pub(crate) limit: u64,
}

pub(crate) fn assert_pressure(pressure: PhysicalRecordPressureEvidence, expected: ExpectedScopePressure) {
    assert_eq!(pressure.basis().store_identity(), expected.store);
    assert_eq!(pressure.dimension(), expected.dimension);
    assert_eq!(pressure.scope(), expected.scope);
    assert_eq!(pressure.requested(), expected.requested);
    assert_eq!(pressure.admitted(), expected.admitted);
    assert_eq!(pressure.limit(), expected.limit);
    assert!(!pressure.effect_may_have_started());
}

pub(crate) fn bytes(value: u64) -> NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

pub(crate) fn frames(value: u32) -> NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}