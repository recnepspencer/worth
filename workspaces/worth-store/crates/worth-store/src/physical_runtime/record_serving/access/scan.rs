use worth_store_physical_format::CurrentPhysicalRecordPlacement;

use super::super::{
    access::manifest_routing::{ManifestLookupFailure, ManifestRangeCursor},
    access::scan_observation::manifest_error,
    access::scan_readmission::{cursor_for, ExternalRecordScanCursor},
    CompletedRecordScan, PhysicalRecordId, PhysicalRecordReader, RecordByteLimit, RecordCountLimit,
    RecordReadLimits, RecordReadObservation, RecordScanCounterSnapshot, RecordScanError,
};

#[path = "scan/batch.rs"]
mod batch;
#[path = "scan/batch_collection.rs"]
mod batch_collection;
#[path = "scan/request_admission.rs"]
mod request_admission;
#[path = "scan/start_position.rs"]
mod start_position;
pub use batch::{RecordScanBatch, RecordScanOutcome, ScannedPhysicalRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordScanRequest {
    cursor: Option<ExternalRecordScanCursor>,
    batch_limit: Option<RecordCountLimit>,
}

impl RecordScanRequest {
    pub const fn from_start() -> Self {
        Self {
            cursor: None,
            batch_limit: None,
        }
    }
    pub const fn resume(cursor: ExternalRecordScanCursor) -> Self {
        Self {
            cursor: Some(cursor),
            batch_limit: None,
        }
    }
    pub const fn with_batch_limit(mut self, limit: RecordCountLimit) -> Self {
        self.batch_limit = Some(limit);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordScanDenial {
    ServingRequiresInspection,
    BatchLimitExceeded,
    BatchMetadataUnavailable,
    ForeignStore,
    StaleRoot,
    FormatMismatch,
    RoutingTreeMismatch,
    CursorPositionNotFound,
    ManifestUnavailable,
    CallerScratchTooSmall { required: u64 },
    RecordRead(super::super::RecordReadDenial),
    RecordStream(super::super::RecordStreamFailureKind),
}

pub struct PhysicalRecordScanSession {
    reader: PhysicalRecordReader,
    cursor: ManifestRangeCursor<'static>,
    pending: Option<CurrentPhysicalRecordPlacement>,
    batch_limit: usize,
    complete: bool,
    total: RecordScanCounterSnapshot,
    _lifecycle: super::super::lifecycle::record_lifecycle::RecordScanSessionLease,
    _allocation: worth_store_buffer_pool::OperationAllocationGrant,
}

impl PhysicalRecordReader {
    pub fn scan(
        mut self,
        request: RecordScanRequest,
    ) -> Result<PhysicalRecordScanSession, RecordScanError> {
        let _call = self.execution.admit_call().map_err(|_| {
            super::scan_observation::scan_error(RecordScanDenial::RecordStream(
                super::super::RecordStreamFailureKind::RuntimeReleased,
            ))
        })?;
        let admission = request_admission::admit_scan_request(&mut self, request)?;
        let positioned = start_position::position_scan_start(
            &self,
            &admission.allocation,
            admission.first,
            &admission.runtime,
        )?;
        let lifecycle = self.lifecycle.scan_session();
        Ok(PhysicalRecordScanSession {
            reader: self,
            cursor: positioned.cursor,
            pending: None,
            batch_limit: admission.batch_limit,
            complete: positioned.complete,
            total: positioned.observation,
            _lifecycle: lifecycle,
            _allocation: admission.allocation,
        })
    }
}

impl PhysicalRecordScanSession {
    pub fn read_next_into<'scratch>(
        &mut self,
        scratch: &'scratch mut [u8],
    ) -> Result<RecordScanOutcome<'scratch>, RecordScanError> {
        let _call = self
            .reader
            .execution
            .admit_call()
            .map_err(|_| RecordScanError {
                denial: RecordScanDenial::RecordStream(
                    super::super::RecordStreamFailureKind::RuntimeReleased,
                ),
                observation: self.total,
            })?;
        self.reader
            .protection
            .require_live()
            .map_err(|_| RecordScanError {
                denial: RecordScanDenial::RecordStream(
                    super::super::RecordStreamFailureKind::RuntimeReleased,
                ),
                observation: self.total,
            })?;
        let runtime = self.reader.runtime.upgrade().ok_or(RecordScanError {
            denial: RecordScanDenial::ServingRequiresInspection,
            observation: self.total,
        })?;
        runtime.health.permit().map_err(|_| RecordScanError {
            denial: RecordScanDenial::ServingRequiresInspection,
            observation: self.total,
        })?;
        if self.complete {
            return Ok(RecordScanOutcome::Completed(CompletedRecordScan {
                observation: self.total,
            }));
        }
        self.collect_next_batch(scratch, &runtime)
    }

    fn take_next_placement(
        &mut self,
    ) -> Result<Option<CurrentPhysicalRecordPlacement>, RecordScanError> {
        if self.pending.is_some() {
            return Ok(self.pending.take());
        }
        let before = self.cursor.counters();
        let next = self.cursor.next(&self._allocation);
        let after = self.cursor.counters();
        self.total.observe_manifest_delta(before, after);
        next.map_err(|failure| {
            let mut error = scan_manifest_error(&self.cursor, failure);
            error.observation = self.total;
            if let Some(runtime) = self.reader.runtime.upgrade() {
                runtime.health.observe_scan_denial(error.denial);
            }
            error
        })
    }

    fn observe_record_read(&mut self, observation: RecordReadObservation) {
        self.total.observe_record_read(observation);
    }
}

fn scan_manifest_error(
    cursor: &ManifestRangeCursor<'_>,
    failure: ManifestLookupFailure,
) -> RecordScanError {
    manifest_error(
        cursor,
        RecordScanDenial::RecordRead(super::locate::failure_classification::manifest_failure(
            failure,
        )),
    )
}
