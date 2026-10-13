use serde::{Deserialize, Serialize};

use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayFailureClass {
    Execution(crate::execution::RelationalExecutionDenialCause),
    MissingCommit,
    MissingAuthoritativeParentClosure,
    BranchMismatch,
    SchemaMismatch,
    UnsupportedReplaySchema,
    AuthoritativeBasisUnavailable,
    ObservableMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayError {
    pub class: ReplayFailureClass,
    pub detail: String,
    pub context: ErrorContext,
}

impl ReplayError {
    pub fn new(class: ReplayFailureClass, detail: impl Into<String>) -> Self {
        Self {
            class,
            detail: detail.into(),
            context: ErrorContext::new(RelationalSubsystem::Replay, ErrorOperation::Replay)
                .with_fix(SuggestedFix::RebuildFromCanonicalArtifacts),
        }
    }
}

impl From<ReplayFailureClass> for ReplayError {
    fn from(value: ReplayFailureClass) -> Self {
        Self::new(value.clone(), format!("{value:?}"))
    }
}
