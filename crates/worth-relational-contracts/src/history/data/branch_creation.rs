use serde::{Deserialize, Serialize};

use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchCreateErrorClass {
    BranchAlreadyExists,
    SourceBranchMissing,
    InvalidTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchCreateError {
    pub class: BranchCreateErrorClass,
    pub detail: String,
    pub context: ErrorContext,
}

impl BranchCreateError {
    fn new(class: BranchCreateErrorClass) -> Self {
        let detail = match class {
            BranchCreateErrorClass::BranchAlreadyExists => "branch already exists".to_string(),
            BranchCreateErrorClass::SourceBranchMissing => "source branch missing".to_string(),
            BranchCreateErrorClass::InvalidTarget => "target branch is invalid".to_string(),
        };
        Self {
            class,
            detail,
            context: ErrorContext::new(RelationalSubsystem::History, ErrorOperation::CreateBranch)
                .with_fix(SuggestedFix::VerifyBranchInputs),
        }
    }

    pub fn branch_already_exists() -> Self {
        Self::new(BranchCreateErrorClass::BranchAlreadyExists)
    }

    pub fn source_branch_missing() -> Self {
        Self::new(BranchCreateErrorClass::SourceBranchMissing)
    }

    pub fn invalid_target() -> Self {
        Self::new(BranchCreateErrorClass::InvalidTarget)
    }
}
