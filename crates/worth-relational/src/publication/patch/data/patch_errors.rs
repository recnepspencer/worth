use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchStreamReadErrorClass {
    UnknownResumePosition,
    InvalidBatchSize,
    RetainedHistoryGap,
    /// The caller's request or an indexed envelope read was refused.
    ReadRequest(crate::execution::RelationalExecutionDenialCause),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchStreamReadError {
    pub class: PatchStreamReadErrorClass,
    pub detail: String,
}
