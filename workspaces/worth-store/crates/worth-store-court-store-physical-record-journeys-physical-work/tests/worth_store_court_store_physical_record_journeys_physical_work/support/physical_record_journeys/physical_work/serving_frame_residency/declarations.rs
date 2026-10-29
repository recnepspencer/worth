pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint, worth_store::physical_runtime::CertificationFrameReadFailure, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordResidencyFailureKind, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadLimits};
pub(crate) use {worth_store_buffer_pool::PhysicalResidencyDenial};
pub(crate) use {worth_store_physical_backend::ArtifactTreeFailureKind, worth_store_physical_backend::MediaOperationRole};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::success};
pub(crate) use {worth_store_testkit_store::durable_publication};

pub(crate) const FRAME_BYTES: u32 = 8;

pub(crate) fn admitted_policy(
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
) -> worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy {
    use {worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope};
    use {worth_store::physical_runtime::PhysicalSpeculativeWorkKind as Speculation};

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
        .pin_leases(nonzero_count(2))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero_bytes(resident_bytes))
        .operation_bytes(nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundRead, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::ForegroundWrite, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Recovery, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Scrub, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Maintenance, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Verification, nonzero_bytes(operation_bytes))
        .scope_bytes(Scope::Blob, nonzero_bytes(operation_bytes))
        .speculative_frames(Speculation::Prefetch, nonzero_count(8))
        .speculative_frames(Speculation::ReadAhead, nonzero_count(8))
        .speculative_frames(Speculation::WriteBehind, nonzero_count(2))
        .admit(format)
        .into_result()
        .unwrap()
}

pub(crate) fn nonzero_bytes(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

pub(crate) fn nonzero_count(value: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(value).unwrap()
}

pub(crate) fn coordinate(offset: u64) -> RecordFrameCoordinate {
    worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, offset, FRAME_BYTES).unwrap()
}

pub(crate) fn pin_and_release(
    residency: &worth_store::physical_runtime::PhysicalResidencyCertification,
    coordinate: RecordFrameCoordinate,
) {
    let frame = residency.pin_exact(coordinate).unwrap();
    assert_eq!(frame.physical_work_count(), 1);
    drop(frame);
}

pub(crate) fn positioned_reads(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead)
}