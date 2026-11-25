use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivedIndexExecutionDenialKind {
    Cause(crate::execution::denial_cause::RelationalExecutionDenialCause),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexExecutionDenial {
    pub kind: DerivedIndexExecutionDenialKind,
    pub partition_identity: Option<u64>,
}
