//! Shared canonical frame bounds and controlled-copy refusal.
use super::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal;
use crate::execution::denial_cause::RelationalExecutionDenialCause;

pub const MAXIMUM_ARTIFACT_BYTES: usize = 256 * 1024 * 1024;

pub enum ControlledFrameCopyDenial {
    Frame(RelationalCanonicalCommitArtifactFrameRefusal),
    Request(RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
}
