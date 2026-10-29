pub(crate) use {worth_store::physical_runtime::AdmittedDirtyFrame, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalResidencyCertification, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkSignalFamily, worth_store::physical_runtime::PhysicalWritebackExecution, worth_store::physical_runtime::PhysicalWritebackFailureCause, worth_store::physical_runtime::PhysicalWritebackSettlement, worth_store::physical_runtime::PreparedPhysicalWriteback};
pub(crate) use {worth_store_buffer_pool::PhysicalResidencyDenial};
pub(crate) use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};
pub(crate) use {worth_store_physical_format::RecordFrameCoordinate};

pub(crate) use {crate::physical_record_journeys::physical_work::speculative_residency::fixture::causal_record, crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate, crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store_with_writebehind};

pub(crate) fn dirty_frame(
    residency: &PhysicalResidencyCertification,
    coordinate: RecordFrameCoordinate,
    fill: u8,
) -> AdmittedDirtyFrame {
    let lease = residency.pin_exact(coordinate).unwrap();
    residency
        .admit_dirty_frame(lease, |_, target| target.fill(fill))
        .unwrap()
}

pub(crate) fn execute_clean(
    residency: &PhysicalResidencyCertification,
    prepared: PreparedPhysicalWriteback,
) -> PhysicalWritebackSettlement {
    let ready = residency.request_writeback(prepared).unwrap();
    let admitted = residency.admit_writeback(ready).unwrap();
    match residency.execute_writeback(admitted).unwrap() {
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(settlement) => settlement,
        worth_store::physical_runtime::PhysicalWritebackExecution::Retryable(_) => {
            panic!("unfaulted writebehind unexpectedly required retry")
        }
        worth_store::physical_runtime::PhysicalWritebackExecution::InspectionRequired(inspection) => {
            panic!(
                "unfaulted writebehind unexpectedly required inspection: {:?}",
                inspection.settlement()
            )
        }
    }
}