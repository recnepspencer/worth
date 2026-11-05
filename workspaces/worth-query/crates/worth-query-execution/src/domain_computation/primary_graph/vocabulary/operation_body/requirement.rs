//! Moved whole from `domain_computation::primary_graph::operation_body::requirement`: the items below and every impl they own.
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementDirection;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementProgram;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyRequirementSource;

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
    QueryContext {
        query: String,
    },
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
    RelatedFieldValues {
        relation: String,
        from: String,
        to: String,
        field: ApplicationFieldBindingLocus,
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
    pub fn new(field: ApplicationFieldBindingLocus, met: bool) -> Self {
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
            ApplicationOperationBodyRequirementSource::QueryContext { query, .. } => {
                WorthQueryOperationBodyRequirementTarget::QueryContext {
                    query: query.name().to_owned(),
                }
            }
            ApplicationOperationBodyRequirementSource::RelatedFieldValues {
                relation,
                from,
                to,
                field,
                ..
            } => WorthQueryOperationBodyRequirementTarget::RelatedFieldValues {
                relation: relation.clone(),
                from: from.clone(),
                to: to.clone(),
                field: field.clone(),
            },
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
