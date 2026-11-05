use serde::{Deserialize, Serialize};

use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};

use crate::publication::bundle::PublicationStage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicationError {
    pub stage: PublicationStage,
    pub detail: String,
    pub context: ErrorContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authored_retirement_refusal: Option<super::AuthoredRetirementRefusal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    invariant_violation: Option<Box<crate::validation::data::InvariantViolation>>,
}

impl PublicationError {
    pub fn authored_retirement_refusal(&self) -> Option<super::AuthoredRetirementRefusal> {
        self.authored_retirement_refusal
    }

    pub(crate) fn authored_retirement(
        cause: super::AuthoredRetirementRefusal,
        subject: crate::transactions::data::RecordRef,
    ) -> Self {
        let mut error = Self::new(
            PublicationStage::BundleAssembly,
            format!("authored cascade retirement: {cause:?}"),
        );
        error.authored_retirement_refusal = Some(cause);
        error.context = error.context.with_records(vec![subject]);
        error
    }

    pub(crate) fn ordinary_metadata_write(
        kind: crate::identity::data::KindId,
        record: Option<crate::transactions::data::RecordRef>,
    ) -> Self {
        let mut error = Self::new(
            PublicationStage::BundleAssembly,
            format!("ordinary write to Native-private kind {kind:?}"),
        );
        error.authored_retirement_refusal =
            Some(super::AuthoredRetirementRefusal::OrdinaryMetadataWrite);
        error.context.affected_records = record.into_iter().collect();
        error
    }
    pub(crate) fn owned_publication(
        cause: super::AuthoredRetirementRefusal,
        member: crate::identity::data::EntityId,
        owner: crate::identity::data::EntityId,
    ) -> Self {
        let mut error =
            Self::authored_retirement(cause, crate::transactions::data::RecordRef::Entity(member));
        error.context = error.context.with_records(vec![
            crate::transactions::data::RecordRef::Entity(member),
            crate::transactions::data::RecordRef::Entity(owner),
        ]);
        error
    }

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
            authored_retirement_refusal: None,
            context: ErrorContext::new(RelationalSubsystem::Publication, ErrorOperation::Publish)
                .with_fix(SuggestedFix::InspectDiagnostics),
        }
    }
}
