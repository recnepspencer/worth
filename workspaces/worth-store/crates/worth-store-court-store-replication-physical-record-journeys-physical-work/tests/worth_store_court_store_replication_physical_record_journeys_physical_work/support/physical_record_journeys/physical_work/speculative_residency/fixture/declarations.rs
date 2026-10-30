pub(crate) use std::path::Path;

pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation,
    worth_store::physical_runtime::PhysicalWorkCausalRecord,
    worth_store::physical_runtime::PhysicalWorkIdentity,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

pub(crate) const FRAME_BYTES: u32 = 8;

pub(crate) fn initialize_store(root: &Path) {
    let (format, placement, access) = configuration();
    let seeded = success(initialize_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    assert!(!seeded.close().residency().requires_inspection());
}

pub(crate) fn open_store(
    root: &Path,
    prefetch_frames: u32,
    read_ahead_frames: u32,
) -> ServingPhysicalRuntime {
    open_store_with_writebehind(root, prefetch_frames, read_ahead_frames, 2)
}

pub(crate) fn open_store_with_writebehind(
    root: &Path,
    prefetch_frames: u32,
    read_ahead_frames: u32,
    writebehind_frames: u32,
) -> ServingPhysicalRuntime {
    let (format, _, access) = configuration();
    success(open_record_store!(media(root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(residency_policy_with_writebehind(
                format,
                prefetch_frames,
                read_ahead_frames,
                writebehind_frames,
            ))
    },))
}

pub(crate) fn residency_policy(
    format: AdmittedPhysicalRecordFormat,
    prefetch_frames: u32,
    read_ahead_frames: u32,
) -> AdmittedPhysicalRecordResidencyPolicy {
    residency_policy_with_writebehind(format, prefetch_frames, read_ahead_frames, 2)
}

pub(crate) fn residency_policy_with_writebehind(
    format: AdmittedPhysicalRecordFormat,
    prefetch_frames: u32,
    read_ahead_frames: u32,
    writebehind_frames: u32,
) -> AdmittedPhysicalRecordResidencyPolicy {
    let resident_bytes = 64 * 1024;
    let metadata_bytes = 16 * 1024;
    let operation_bytes = 4 * 1024 * 1024;
    worth_store::physical_runtime::PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero_bytes(
            operation_bytes + metadata_bytes + (2 * resident_bytes),
        ))
        .resident_bytes(nonzero_bytes(resident_bytes))
        .metadata_bytes(nonzero_bytes(metadata_bytes))
        .frame_entries(nonzero_count(8))
        .pinned_frames(nonzero_count(8))
        .pin_leases(nonzero_count(8))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero_bytes(resident_bytes))
        .operation_bytes(nonzero_bytes(operation_bytes))
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
            nonzero_bytes(operation_bytes),
        )
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite,
            nonzero_bytes(operation_bytes),
        )
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Recovery,
            nonzero_bytes(operation_bytes),
        )
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Scrub,
            nonzero_bytes(operation_bytes),
        )
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance,
            nonzero_bytes(operation_bytes),
        )
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Verification,
            nonzero_bytes(operation_bytes),
        )
        .scope_bytes(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::Blob,
            nonzero_bytes(operation_bytes),
        )
        .speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch,
            nonzero_count(prefetch_frames),
        )
        .speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead,
            nonzero_count(read_ahead_frames),
        )
        .speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind,
            nonzero_count(writebehind_frames),
        )
        .admit(format)
        .into_result()
        .unwrap()
}

pub(crate) fn coordinate(ordinal: u64) -> RecordFrameCoordinate {
    worth_store_physical_format::RecordFrameCoordinate::new(
        worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
        ordinal * u64::from(FRAME_BYTES),
        FRAME_BYTES,
    )
    .unwrap()
}

pub(crate) fn positioned_reads(serving: &ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
}

pub(crate) fn causal_record(
    serving: &ServingPhysicalRuntime,
    identity: PhysicalWorkIdentity,
) -> PhysicalWorkCausalRecord {
    serving
        .physical_work_observer()
        .causal()
        .records()
        .iter()
        .copied()
        .find(|record| record.identity() == identity)
        .expect("reported speculative work identity must exist in the causal ledger")
}

pub(crate) fn nonzero_bytes(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

pub(crate) fn nonzero_count(value: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}
