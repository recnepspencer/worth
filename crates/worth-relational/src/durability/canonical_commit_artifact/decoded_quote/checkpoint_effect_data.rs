//! The funded semantic result and creation evidence from one selected checkpoint.
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub(crate) struct CheckpointEffectData {
    pub(crate) semantic: Arc<RelationalCommitSemanticResult>,
    pub(crate) creations: Vec<PublishedAuthoritativeRecordPatch>,
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}
