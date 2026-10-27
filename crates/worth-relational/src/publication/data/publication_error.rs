use serde::{Deserialize, Serialize};

use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};

use crate::publication::bundle::PublicationStage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationError {
    pub stage: PublicationStage,
    pub detail: String,
    pub context: ErrorContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    invariant_violation: Option<Box<crate::validation::data::InvariantViolation>>,
}

impl PublicationError {
    pub fn from_invariant_violation(
        stage: PublicationStage,
        violation: crate::validation::data::InvariantViolation,
    ) -> Self {
        let mut error = Self::new(stage, violation.detail.clone());
        error.invariant_violation = Some(Box::new(violation));
        error
    }

    /// Original native invariant evidence, independent of the display detail.
    pub fn invariant_violation(&self) -> Option<&crate::validation::data::InvariantViolation> {
        self.invariant_violation.as_deref()
    }

    pub fn into_invariant_violation(self) -> Option<crate::validation::data::InvariantViolation> {
        self.invariant_violation.map(|violation| *violation)
    }

    pub fn new(stage: PublicationStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
            invariant_violation: None,
            context: ErrorContext::new(RelationalSubsystem::Publication, ErrorOperation::Publish)
                .with_fix(SuggestedFix::InspectDiagnostics),
        }
    }
}
