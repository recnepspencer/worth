use worth_store_physical_format::{DurablePhysicalRootManifest, RecordAnchorAdvance};

use crate::physical_runtime::record_serving::{
    publication::batch::RecordBatchAnchorAdvance, PreparedPhysicalRootProjection,
};
use crate::physical_runtime::{RecordAppendDenial, RecordAppendError};

/// The anchor request of a materialized batch with the digest of the record it
/// names. Captured before the batch is consumed by placement planning.
pub(super) struct PlannedRecordAnchorRequest {
    request: RecordBatchAnchorAdvance,
    record_payload_sha256: [u8; 32],
}

impl PlannedRecordAnchorRequest {
    pub(super) fn capture(
        batch: &crate::physical_runtime::RecordAppendBatch,
    ) -> Result<Option<Self>, RecordAppendError> {
        let Some(request) = batch.anchor_advance else {
            return Ok(None);
        };
        let record_payload_sha256 =
            batch
                .prepared_record_sha256(request.record_index)
                .ok_or(RecordAppendError::Denied(
                    RecordAppendDenial::InvalidRecordAnchorAdvance,
                ))?;
        Ok(Some(Self {
            request,
            record_payload_sha256,
        }))
    }

    /// Checks the advance against the source root and binds it to the planned
    /// record identity. Any refusal precedes the WAL append.
    pub(super) fn bind(
        self,
        source_root: &DurablePhysicalRootManifest,
        root: &mut PreparedPhysicalRootProjection,
    ) -> Result<(), RecordAppendError> {
        let invalid = || RecordAppendError::Denied(RecordAppendDenial::InvalidRecordAnchorAdvance);
        let record = root
            .recovery_record_identities()
            .nth(usize::from(self.request.record_index))
            .ok_or_else(invalid)?;
        let advance = RecordAnchorAdvance::new(
            self.request.name,
            self.request.expected_generation,
            record,
            self.record_payload_sha256,
        )
        .ok_or_else(invalid)?;
        source_root
            .record_anchors()
            .advanced(advance)
            .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::RecordAnchorConflict))?;
        root.set_record_anchor_advance(advance);
        Ok(())
    }
}
