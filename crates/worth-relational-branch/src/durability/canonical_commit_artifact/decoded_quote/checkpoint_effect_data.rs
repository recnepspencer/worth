//! The funded semantic result and creation evidence from one selected checkpoint.
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug)]
pub struct CheckpointEffectData {
    pub semantic: Arc<RelationalCommitSemanticResult>,
    pub creations: Vec<PublishedAuthoritativeRecordPatch>,
    pub _memory: Option<ExecutionMemoryReservation>,
}
