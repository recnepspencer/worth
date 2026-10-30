pub(crate) use {
    worth_store::physical_runtime::AdmittedDirtyFrame,
    worth_store::physical_runtime::PhysicalResidencyCertification,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkRecoveryDisposition,
    worth_store::physical_runtime::PhysicalWorkRetryScheduleOutcome,
    worth_store::physical_runtime::PhysicalWritebackExecution,
};
pub(crate) use {
    worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement,
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaOperationRole,
};
pub(crate) use {
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::RecordFrameCoordinate,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

pub(crate) const EXPECTED_WRITEBACK: [u8; 8] = [0xc6, 0x51, 0xfa, 0x17, 0x90, 0x2d, 0x33, 0x74];

pub(crate) fn dirty_frame(
    residency: &PhysicalResidencyCertification,
    coordinate: RecordFrameCoordinate,
) -> AdmittedDirtyFrame {
    let lease = residency.pin_exact(coordinate).unwrap();
    residency
        .admit_dirty_frame(lease, |_, target| {
            target.copy_from_slice(&EXPECTED_WRITEBACK);
        })
        .unwrap()
}

pub(crate) fn artifact_range(root: &std::path::Path, coordinate: RecordFrameCoordinate) -> Vec<u8> {
    let bytes = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let start = coordinate.offset() as usize;
    bytes[start..start + coordinate.length() as usize].to_vec()
}

pub(crate) fn clock_advance(tick: u64) -> worth_signal::facade::ClockAdvanceRequest {
    worth_signal::facade::ClockAdvanceRequest::new(
        worth_signal::facade::ClockDomain::MonotonicExecution,
        worth_signal::facade::ClockTick::new(tick),
    )
}

pub(crate) fn completed_writes(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> u64 {
    serving
        .media_counters()
        .completed_operations_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
}

pub(crate) fn denied_writes(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
) -> u64 {
    serving
        .media_counters()
        .denied_before_effect_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
}
