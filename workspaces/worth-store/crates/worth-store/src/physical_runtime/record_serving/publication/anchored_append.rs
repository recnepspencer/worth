//! One record that advances one root anchor, submitted as one durable root
//! decision.
//!
//! Every Store-owned anchored door (the ordered journals and the
//! anchored-record door) commits exactly this shape: a single framed record,
//! the anchor advance naming it and expecting the generation the door observed,
//! a door-specific idempotency material, and the ordinary durable append path.
//! The doors differ only in their frames, their admission checks and how they
//! name the answer.

use worth_store_physical_format::RecordAnchorName;

use super::batch::RecordBatchAnchorAdvance;
use super::durable_append::{DurableRecordAppend, DurableRecordAppendFailure};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, RecordAppendBatch,
};

/// A framed record and the anchor movement it requests.
pub(in crate::physical_runtime) struct AnchoredRecordAppend {
    pub(in crate::physical_runtime) record: Vec<u8>,
    pub(in crate::physical_runtime) anchor: RecordAnchorName,
    /// The anchor generation the door observed; zero when absent.
    pub(in crate::physical_runtime) expected_generation: u64,
    /// Door-specific, domain-separated idempotency material.
    pub(in crate::physical_runtime) idempotency: [u8; 32],
    pub(in crate::physical_runtime) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime) deadline: PhysicalMutationDeadline,
}

impl AnchoredRecordAppend {
    pub(in crate::physical_runtime) fn into_durable_append(
        self,
    ) -> Result<DurableRecordAppend, DurableRecordAppendFailure> {
        let batch = RecordAppendBatch::builder()
            .push_owned(self.record)
            .build()
            .and_then(|batch| {
                batch.with_anchor_advance(RecordBatchAnchorAdvance {
                    name: self.anchor,
                    expected_generation: self.expected_generation,
                    record_index: 0,
                })
            })
            .map_err(DurableRecordAppendFailure::Batch)?;
        Ok(DurableRecordAppend {
            batch,
            idempotency: self.idempotency,
            placement: self.placement,
            deadline: self.deadline,
        })
    }
}
