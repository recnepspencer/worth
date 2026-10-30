pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalResidencyAllocationSnapshot,
    worth_store::physical_runtime::PhysicalResidencyCounterSnapshot,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::PhysicalResidencyRetryPosture,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadLimits,
};

pub(crate) use worth_store_testkit_store::durable_publication;
pub(crate) use {
    crate::physical_record_journeys::physical_work::residency_pressure_projection::nonzero,
    crate::physical_record_journeys::physical_work::residency_pressure_projection::nonzero_count,
};
pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) fn one_page_foreground_read_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope;
    use worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Kind;

    let page = u64::from(format.declaration().page_size().bytes());
    // Reopen authenticates C11 routing/free-space metadata before the read
    // experiment. Its bounded Recovery scratch is separate from the one-page
    // ForegroundRead scope whose contention this test exercises.
    let metadata = page * 16;
    let resident = page * 4;
    // The reopened WAL roster needs 71,704 bytes in this two-record world;
    // usable Recovery bytes exclude the policy's progress headroom. Six pages
    // admit that bootstrap work without changing the one-page read contention.
    let recovery = page * 6;
    let mut builder = worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero(metadata + resident + recovery + page))
        .resident_bytes(nonzero(resident))
        .metadata_bytes(nonzero(metadata))
        .frame_entries(nonzero_count(4))
        .pinned_frames(nonzero_count(4))
        .pin_leases(nonzero_count(4))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero(page))
        .operation_bytes(nonzero(recovery));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        builder = builder.scope_bytes(
            scope,
            nonzero(if scope == Scope::Recovery {
                recovery
            } else {
                page
            }),
        );
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        builder = builder.speculative_frames(kind, nonzero_count(2));
    }
    builder.admit(format).into_result().unwrap()
}
