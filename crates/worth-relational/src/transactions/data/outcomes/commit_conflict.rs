use crate::diagnostics::data::DiagnosticCode;
use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};
use serde::{Deserialize, Serialize};

use super::ConflictClass;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitConflict {
    pub class: ConflictClass,
    pub code: DiagnosticCode,
    pub detail: String,
    pub context: ErrorContext,
}

impl CommitConflict {
    pub(crate) fn new(class: ConflictClass) -> Self {
        let code = class.code();
        let detail = class.detail();
        Self {
            class,
            code,
            detail,
            context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Validate)
                .with_fix(SuggestedFix::InspectDiagnostics),
        }
    }

    /// Original physical owner cause; quota/conflict variants carry none.
    pub fn allocation_denial(&self) -> Option<&worth_execution::ExecutionAllocationDenial> {
        match &self.class {
            ConflictClass::ExecutionAllocationDenied { denial } => Some(denial),
            _ => None,
        }
    }
    pub fn code(&self) -> DiagnosticCode {
        self.code
    }

    pub fn detail(&self) -> String {
        self.detail.clone()
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }
}
