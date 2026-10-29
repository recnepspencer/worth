pub(crate) use {std::cell::Cell};

pub(crate) use {worth_store::physical_runtime::AdmittedDirtyFrame, worth_store::physical_runtime::CertificationFrameReadFailure, worth_store::physical_runtime::CertificationFrameWorkFailure, worth_store::physical_runtime::PhysicalDirtyTransitionFailure, worth_store::physical_runtime::PhysicalResidencyAllocationBoundaryKind, worth_store::physical_runtime::PhysicalResidencyCertification, worth_store::physical_runtime::PhysicalWorkPreEffectDenial, worth_store::physical_runtime::PhysicalWritebackFailureCause};
pub(crate) use {worth_store_buffer_pool::PhysicalResidencyDimension};
pub(crate) use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};
pub(crate) use {worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFrameCoordinate};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

pub(crate) const REPLACEMENT: [u8; 8] = [0x71, 0xc6, 0x93, 0x28, 0x45, 0xba, 0xd0, 0x1f];

pub(crate) fn dirty_frame(residency: &PhysicalResidencyCertification) -> AdmittedDirtyFrame {
    residency
        .admit_dirty_frame(residency.pin_exact(coordinate()).unwrap(), |_, target| {
            target.copy_from_slice(&REPLACEMENT);
        })
        .unwrap()
}

pub(crate) fn coordinate() -> RecordFrameCoordinate {
    worth_store_physical_format::RecordFrameCoordinate::new(worth_store_physical_format::RecordArtifactFile::BootstrapCatalog, 8, 8).unwrap()
}