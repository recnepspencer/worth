//! Descriptive result of the same installed precondition used by native admission.
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementDirection, ApplicationOperationBodyRequirementProgram,
    ApplicationOperationBodyRequirementSource,
};
mod evaluation;
mod value;
pub(in crate::domain_computation::primary_graph) use evaluation::evaluate_requirements;
pub(in crate::domain_computation::primary_graph) use value::{
    field_value, PreparedRequirementOperand,
};

fn expression_denial(
    cause: worth_foundational::expression_api::ExpressionDenial,
) -> crate::domain_computation::primary_graph::HandlerExecutionDenial {
    crate::domain_computation::primary_graph::HandlerExecutionDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::body_requirement_expression("operation body requirement", cause))
}

/// The current observation of one installed operation-body precondition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationBodyRequirementStatus {
    /// The observed field satisfies the installed precondition.
    Met,
    /// The observed field does not satisfy the installed precondition.
    Unmet,
    /// The authored applicability expression is false at this observation.
    NotApplicable,
}

/// The native source the supplier declared as this requirement's guidance target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationBodyRequirementTarget {
    SignedEvidence(
        worth_query_declaration::facade::application_schema::ApplicationSigningEvidenceSource,
    ),
    RelatedSignedEvidence {
        relation: String,
        from: String,
        to: String,
        evidence:
            worth_query_declaration::facade::application_schema::ApplicationSigningEvidenceSource,
    },
    Field(ApplicationFieldBindingLocus),
    RelatedItems {
        relation: String,
        from: String,
        to: String,
        direction: ApplicationOperationBodyRequirementDirection,
    },
}

/// A descriptive finding for one installed body field; it grants no mutation authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationBodyRequirementFinding {
    target: WorthQueryOperationBodyRequirementTarget,
    identity: Option<String>,
    status: WorthQueryOperationBodyRequirementStatus,
}

impl WorthQueryOperationBodyRequirementFinding {
    pub(in crate::domain_computation::primary_graph) fn new(
        field: ApplicationFieldBindingLocus,
        met: bool,
    ) -> Self {
        Self {
            target: WorthQueryOperationBodyRequirementTarget::Field(field),
            identity: None,
            status: if met {
                WorthQueryOperationBodyRequirementStatus::Met
            } else {
                WorthQueryOperationBodyRequirementStatus::Unmet
            },
        }
    }

    pub fn field(&self) -> Option<&ApplicationFieldBindingLocus> {
        match &self.target {
            WorthQueryOperationBodyRequirementTarget::Field(field) => Some(field),
            _ => None,
        }
    }

    pub fn identity(&self) -> Option<&str> {
        self.identity.as_deref()
    }
    pub fn target(&self) -> &WorthQueryOperationBodyRequirementTarget {
        &self.target
    }
    pub(in crate::domain_computation::primary_graph) fn evaluated(
        requirement: &ApplicationOperationBodyRequirementProgram,
        status: WorthQueryOperationBodyRequirementStatus,
    ) -> Self {
        let target = match requirement.target() {
            ApplicationOperationBodyRequirementSource::SignedEvidence(source) => {
                WorthQueryOperationBodyRequirementTarget::SignedEvidence(source.clone())
            }
            ApplicationOperationBodyRequirementSource::RelatedSignedEvidence {
                relation,
                from,
                to,
                evidence,
            } => WorthQueryOperationBodyRequirementTarget::RelatedSignedEvidence {
                relation: relation.clone(),
                from: from.clone(),
                to: to.clone(),
                evidence: evidence.clone(),
            },
            ApplicationOperationBodyRequirementSource::Field { field, .. }
            | ApplicationOperationBodyRequirementSource::PrincipalIdentity { field, .. } => {
                WorthQueryOperationBodyRequirementTarget::Field(field.clone())
            }
            ApplicationOperationBodyRequirementSource::RelatedItemCount {
                relation,
                from,
                to,
                direction,
            } => WorthQueryOperationBodyRequirementTarget::RelatedItems {
                relation: relation.clone(),
                from: from.clone(),
                to: to.clone(),
                direction: *direction,
            },
        };
        Self {
            identity: Some(requirement.identity().to_owned()),
            target,
            status,
        }
    }

    pub const fn status(&self) -> WorthQueryOperationBodyRequirementStatus {
        self.status
    }
}
