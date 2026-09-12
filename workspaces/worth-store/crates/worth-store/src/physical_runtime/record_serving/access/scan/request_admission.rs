use worth_store_physical_format::PersistedRecordIdentity;

use super::{PhysicalRecordReader, RecordScanDenial, RecordScanError, RecordScanRequest};
use crate::physical_runtime::record_serving::access::{
    scan_observation::scan_error, scan_readmission::readmit_cursor,
};

pub(super) struct AdmittedScanRequest {
    pub(super) runtime: std::sync::Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
    pub(super) first: Option<PersistedRecordIdentity>,
    pub(super) batch_limit: usize,
    pub(super) allocation: worth_store_buffer_pool::OperationAllocationGrant,
}

pub(super) fn admit_scan_request(
    reader: &mut PhysicalRecordReader,
    request: RecordScanRequest,
) -> Result<AdmittedScanRequest, RecordScanError> {
    reader.protection.require_live().map_err(|_| {
        scan_error(RecordScanDenial::RecordStream(
            crate::physical_runtime::RecordStreamFailureKind::RuntimeReleased,
        ))
    })?;
    let runtime = reader
        .runtime
        .upgrade()
        .ok_or_else(|| scan_error(RecordScanDenial::ServingRequiresInspection))?;
    runtime
        .health
        .permit()
        .map_err(|_| scan_error(RecordScanDenial::ServingRequiresInspection))?;
    let requested = request
        .batch_limit
        .unwrap_or(reader.access.scan_limit())
        .get();
    if requested > reader.access.scan_limit().get() {
        return Err(scan_error(RecordScanDenial::BatchLimitExceeded));
    }
    let allocation = begin_scan_allocation(reader, requested)?;
    reader.residency = reader.residency.clone().for_scan();
    let first = readmit_cursor(reader, request.cursor)?;
    Ok(AdmittedScanRequest {
        runtime,
        first,
        batch_limit: requested as usize,
        allocation,
    })
}

fn begin_scan_allocation(
    reader: &PhysicalRecordReader,
    requested: u32,
) -> Result<worth_store_buffer_pool::OperationAllocationGrant, RecordScanError> {
    let operation_bytes = u64::from(reader.format.declaration().page_size().bytes())
        .saturating_add(
            u64::from(requested)
                .saturating_mul(std::mem::size_of::<super::ScannedPhysicalRecord>() as u64),
        );
    reader
        .residency
        .begin_operation(
            worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundRead,
            std::num::NonZeroU64::new(operation_bytes)
                .expect("an admitted scan requests nonzero operation bytes"),
        )
        .map_err(|reason| {
            scan_error(RecordScanDenial::RecordRead(
                crate::physical_runtime::record_serving::RecordReadDenial::from_residency(reason),
            ))
        })
}
