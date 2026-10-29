pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint, worth_store::physical_runtime::AdmittedDirtyFrame, worth_store::physical_runtime::PhysicalEffectObligation, worth_store::physical_runtime::PhysicalResidencyCertification, worth_store::physical_runtime::PhysicalSignalSettlementOutcome, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkPreEffectDenial, worth_store::physical_runtime::PhysicalWorkRecoveryDisposition, worth_store::physical_runtime::PhysicalWritebackExecution, worth_store::physical_runtime::PhysicalWritebackFailureCause};
pub(crate) use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement, worth_store_physical_backend::MediaFaultDirective, worth_store_physical_backend::MediaOperationRole};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_write_fault, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub(crate) const WRITEBACK: [u8; 8] = [0x51, 0xc6, 0x7a, 0x19, 0x84, 0x2f, 0xd0, 0x33];

pub(crate) fn dirty_frame(
    residency: &PhysicalResidencyCertification,
    coordinate: RecordFrameCoordinate,
) -> AdmittedDirtyFrame {
    let lease = residency.pin_exact(coordinate).unwrap();
    residency
        .admit_dirty_frame(lease, |_, target| target.copy_from_slice(&WRITEBACK))
        .unwrap()
}

pub(crate) fn writeback_coordinate() -> RecordFrameCoordinate {
    worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap()
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