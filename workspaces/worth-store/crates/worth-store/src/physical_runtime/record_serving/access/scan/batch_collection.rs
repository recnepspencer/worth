use worth_store_physical_format::CurrentPhysicalRecordPlacement;

use super::{
    cursor_for, CompletedRecordScan, PhysicalRecordId, PhysicalRecordScanSession, RecordByteLimit,
    RecordReadLimits, RecordReadObservation, RecordScanBatch, RecordScanDenial, RecordScanError,
    RecordScanOutcome, ScannedPhysicalRecord,
};
use crate::physical_runtime::{
    instance::PhysicalStoreWorkRuntime, RecordReadDenial, RecordStreamFailureKind,
};

struct ScanBatchAssembly {
    records: Vec<ScannedPhysicalRecord>,
    used: usize,
}

struct PlacementReadShape {
    record: PhysicalRecordId,
    payload_limit: RecordByteLimit,
    payload_bytes: usize,
}

impl PhysicalRecordScanSession {
    pub(super) fn collect_next_batch<'scratch>(
        &mut self,
        scratch: &'scratch mut [u8],
        runtime: &PhysicalStoreWorkRuntime,
    ) -> Result<RecordScanOutcome<'scratch>, RecordScanError> {
        let mut batch = ScanBatchAssembly::reserve(self.batch_limit, self.total)?;
        while !batch.is_full(self.batch_limit) {
            let Some(placement) = self.take_next_placement()? else {
                self.complete = true;
                break;
            };
            if !self.collect_placement(&mut batch, placement, scratch, runtime)? {
                break;
            }
        }
        self.finish_batch(batch, scratch)
    }

    fn collect_placement(
        &mut self,
        batch: &mut ScanBatchAssembly,
        placement: CurrentPhysicalRecordPlacement,
        scratch: &mut [u8],
        runtime: &PhysicalStoreWorkRuntime,
    ) -> Result<bool, RecordScanError> {
        let shape = self.read_shape(placement)?;
        if placement.payload_bytes() > self.payload_limit {
            self.total.records = self.total.records.saturating_add(1);
            batch.push(shape.record, None, placement);
            return Ok(true);
        }
        if shape.payload_bytes > scratch.len().saturating_sub(batch.used) {
            self.pending = Some(placement);
            if batch.records.is_empty() {
                return Err(self.error(RecordScanDenial::CallerScratchTooSmall {
                    required: placement.payload_bytes(),
                }));
            }
            return Ok(false);
        }
        let start = batch.used;
        let end = start + shape.payload_bytes;
        let record = shape.record;
        self.read_payload(placement, shape, &mut scratch[start..end], runtime)?;
        batch.used = end;
        self.total.records += 1;
        batch.push(record, Some(start..end), placement);
        Ok(true)
    }

    fn read_shape(
        &self,
        placement: CurrentPhysicalRecordPlacement,
    ) -> Result<PlacementReadShape, RecordScanError> {
        let damaged = || {
            self.error(RecordScanDenial::RecordRead(
                RecordReadDenial::ArtifactDamaged,
            ))
        };
        let payload_limit = u32::try_from(placement.payload_bytes())
            .ok()
            .and_then(|bytes| RecordByteLimit::new(bytes.max(1)))
            .ok_or_else(damaged)?;
        let payload_bytes = usize::try_from(placement.payload_bytes()).map_err(|_| damaged())?;
        Ok(PlacementReadShape {
            record: PhysicalRecordId::from_persisted(placement.record()),
            payload_limit,
            payload_bytes,
        })
    }

    fn read_payload(
        &mut self,
        placement: CurrentPhysicalRecordPlacement,
        shape: PlacementReadShape,
        destination: &mut [u8],
        runtime: &PhysicalStoreWorkRuntime,
    ) -> Result<(), RecordScanError> {
        let mut session = self
            .reader
            .open_known_placement(
                shape.record,
                placement,
                RecordReadLimits::new(shape.payload_limit),
                RecordReadObservation::default(),
            )
            .map_err(|error| {
                self.observe_record_read(error.observation());
                self.error(RecordScanDenial::RecordRead(error.denial()))
            })?;
        let mut used = 0;
        while used < destination.len() {
            let count = session
                .read_next(&mut destination[used..])
                .map_err(|failure| {
                    self.observe_record_read(session.observation());
                    self.error(RecordScanDenial::RecordStream(failure.kind()))
                })?;
            if count == 0 {
                self.observe_record_read(session.observation());
                runtime
                    .health
                    .observe_stream_failure(RecordStreamFailureKind::ArtifactDamaged);
                return Err(self.error(RecordScanDenial::RecordStream(
                    RecordStreamFailureKind::ArtifactDamaged,
                )));
            }
            used += count;
        }
        self.observe_record_read(session.observation());
        Ok(())
    }

    fn finish_batch<'scratch>(
        &mut self,
        batch: ScanBatchAssembly,
        scratch: &'scratch [u8],
    ) -> Result<RecordScanOutcome<'scratch>, RecordScanError> {
        if batch.is_full(self.batch_limit) {
            self.pending = self.take_next_placement()?;
            self.complete = self.pending.is_none();
        }
        let Some(last) = batch.records.last() else {
            self.complete = true;
            return Ok(RecordScanOutcome::Completed(CompletedRecordScan {
                observation: self.total,
            }));
        };
        let end_cursor = cursor_for(&self.reader, &self.reader.current_root, last.record_id());
        Ok(RecordScanOutcome::Batch(RecordScanBatch::new(
            &scratch[..batch.used],
            batch.records,
            end_cursor,
            self.complete,
            self.total,
        )))
    }

    fn error(&self, denial: RecordScanDenial) -> RecordScanError {
        RecordScanError {
            denial,
            observation: self.total,
        }
    }
}

impl ScanBatchAssembly {
    fn reserve(
        batch_limit: usize,
        observation: super::RecordScanCounterSnapshot,
    ) -> Result<Self, RecordScanError> {
        let mut records = Vec::new();
        records
            .try_reserve_exact(batch_limit)
            .map_err(|_| RecordScanError {
                denial: RecordScanDenial::BatchMetadataUnavailable,
                observation,
            })?;
        Ok(Self { records, used: 0 })
    }

    fn is_full(&self, batch_limit: usize) -> bool {
        self.records.len() == batch_limit
    }

    fn push(
        &mut self,
        record: PhysicalRecordId,
        payload: Option<std::ops::Range<usize>>,
        placement: CurrentPhysicalRecordPlacement,
    ) {
        self.records.push(ScannedPhysicalRecord::new(
            record,
            payload,
            placement.payload_bytes(),
            placement.route_metadata(),
        ));
    }
}
