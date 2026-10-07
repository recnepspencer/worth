//! Existing single-node callers keep the payload-only callback; epoch callers
//! share the same preparation meter through the work-aware owner entry point.
use super::retained_node_edit::{
    RetainedNodeEditDenial, RetainedNodeEditPreparation, RetainedNodePayload,
};
use super::NodeArena;
use crate::data::retained_storage::{
    RetainedStorageCharge, RetainedStoragePreparation, SignalConditionalRetentionLedger,
};
use std::sync::Arc;

impl NodeArena {
    pub(crate) fn prepare_retained_node_edits<R>(
        &self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        indices: &[usize],
        maximum: RetainedStorageCharge,
        work: &mut RetainedStoragePreparation,
        edit: impl FnOnce(&mut [RetainedNodePayload]) -> R,
    ) -> Result<RetainedNodeEditPreparation<R>, RetainedNodeEditDenial> {
        self.prepare_retained_node_edits_with_work(ledger, indices, maximum, work, |payloads, _| {
            edit(payloads)
        })
    }
}
