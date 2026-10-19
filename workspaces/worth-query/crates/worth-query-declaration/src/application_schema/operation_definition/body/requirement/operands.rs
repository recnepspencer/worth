//! Authority-free native operand selectors; schema admission binds them.
use super::ApplicationOperationBodyRequirementDenial;
use crate::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationFieldPresence,
        ApplicationOperationDecisionReadTarget,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
use worth_foundational::{
    expression_api::{ExpressionType, IntegerType},
    facade::ScalarAspectType,
};

/// The scope endpoint from which a collection is read.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyRequirementDirection {
    Outgoing,
    Incoming,
}

/// A field observation or distinct related-item count, never a caller value.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyRequirementSource {
    /// Previously admitted values; the exact query declares their recursive
    /// consumption contract, not an operation-side graph reread permission.
    QueryContext {
        mutation_binding: String,
        query: Box<crate::application_query::ErasedApplicationQueryDefinition>,
    },
    /// True when at least one current native attestation matches this exact producer.
    SignedEvidence(crate::application_schema::ApplicationSigningEvidenceSource),
    /// Select exactly one outgoing subject, then observe that producer's current evidence.
    RelatedSignedEvidence {
        relation: String,
        from: String,
        to: String,
        evidence: crate::application_schema::ApplicationSigningEvidenceSource,
    },
    Field {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        presence: ApplicationFieldPresence,
        presence_only: bool,
    },
    RelatedItemCount {
        relation: String,
        from: String,
        to: String,
        direction: ApplicationOperationBodyRequirementDirection,
    },
    /// Values from each distinct live outgoing endpoint, in native identity order.
    RelatedFieldValues {
        relation: String,
        from: String,
        to: String,
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        presence: ApplicationFieldPresence,
    },
    PrincipalIdentity {
        scope: String,
        mutation_binding: String,
        principal_binding: String,
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
    },
}

/// One named source in a requirement's Foundation operand schema.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyRequirementOperand {
    name: String,
    source: ApplicationOperationBodyRequirementSource,
}

impl ApplicationOperationBodyRequirementOperand {
    pub fn from_untrusted(name: String, source: ApplicationOperationBodyRequirementSource) -> Self {
        Self { name, source }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn source(&self) -> &ApplicationOperationBodyRequirementSource {
        &self.source
    }
}

impl ApplicationOperationBodyRequirementSource {
    pub fn scope_entity(&self) -> &str {
        match self {
            Self::QueryContext { query, .. } => query.scope_entity(),
            Self::SignedEvidence(source) => &source.lifecycle.source_entity,
            Self::RelatedSignedEvidence { from, .. } => from,
            Self::RelatedFieldValues { from, .. } => from,
            Self::Field { field, .. } => field.entity(),
            Self::PrincipalIdentity { scope, .. } => scope,
            Self::RelatedItemCount {
                from,
                to,
                direction,
                ..
            } => match direction {
                ApplicationOperationBodyRequirementDirection::Outgoing => from,
                ApplicationOperationBodyRequirementDirection::Incoming => to,
            },
        }
    }
    pub fn decision_reads(&self) -> Vec<ApplicationOperationDecisionReadTarget> {
        match self {
            // Source-expectation facts, not fresh raw reads, carry these dependencies.
            Self::QueryContext { .. } => Vec::new(),
            Self::RelatedFieldValues {
                relation,
                from,
                to,
                field,
                ..
            } => vec![
                ApplicationOperationDecisionReadTarget::Relation {
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                },
                ApplicationOperationDecisionReadTarget::Entity {
                    entity: from.clone(),
                },
                ApplicationOperationDecisionReadTarget::Entity { entity: to.clone() },
                ApplicationOperationDecisionReadTarget::Field {
                    entity: field.entity().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                },
            ],
            Self::SignedEvidence(source) => source.decision_reads(),
            Self::RelatedSignedEvidence {
                relation,
                from,
                to,
                evidence,
            } => {
                let mut reads = evidence.decision_reads();
                reads.extend([
                    ApplicationOperationDecisionReadTarget::Relation {
                        relation: relation.clone(),
                        from: from.clone(),
                        to: to.clone(),
                    },
                    ApplicationOperationDecisionReadTarget::Entity {
                        entity: from.clone(),
                    },
                    ApplicationOperationDecisionReadTarget::Entity { entity: to.clone() },
                ]);
                reads
            }
            Self::Field { field, .. } | Self::PrincipalIdentity { field, .. } => {
                vec![ApplicationOperationDecisionReadTarget::Field {
                    entity: field.entity().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                }]
            }
            Self::RelatedItemCount {
                relation, from, to, ..
            } => vec![
                ApplicationOperationDecisionReadTarget::Relation {
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                },
                ApplicationOperationDecisionReadTarget::Entity {
                    entity: from.clone(),
                },
                ApplicationOperationDecisionReadTarget::Entity { entity: to.clone() },
            ],
        }
    }
    pub fn expression_type(
        &self,
    ) -> Result<ExpressionType, ApplicationOperationBodyRequirementDenial> {
        let (scalar_family, presence, presence_only) = match self {
            Self::QueryContext { query, .. } => {
                if !query.parameters().is_empty()
                    || query.cardinality()
                        != crate::application_query::ApplicationQueryCardinality::ExactlyOne
                {
                    return Err(ApplicationOperationBodyRequirementDenial::UnsupportedQueryContext);
                }
                return Ok(query
                    .result_shape()
                    .append_expression_context(
                        worth_foundational::expression_api::ExpressionSchema::builder(),
                    )?
                    .1);
            }
            Self::RelatedFieldValues {
                scalar_family,
                presence,
                ..
            } => {
                let element = scalar_type(*scalar_family, *presence)?;
                return Ok(ExpressionType::list(element));
            }
            Self::SignedEvidence(_) | Self::RelatedSignedEvidence { .. } => {
                return Ok(ExpressionType::Bool)
            }
            Self::Field {
                scalar_family,
                presence,
                presence_only,
                ..
            } => (*scalar_family, *presence, *presence_only),
            Self::PrincipalIdentity { scalar_family, .. } => {
                (*scalar_family, ApplicationFieldPresence::Required, false)
            }
            Self::RelatedItemCount { .. } => {
                return Ok(ExpressionType::Integer(IntegerType::UInt64))
            }
        };
        if presence_only {
            return Ok(ExpressionType::Bool);
        }
        scalar_type(scalar_family, presence)
    }
}

fn scalar_type(
    scalar_family: ScalarAspectType,
    presence: ApplicationFieldPresence,
) -> Result<ExpressionType, ApplicationOperationBodyRequirementDenial> {
    let ty = ExpressionType::from_scalar_family(scalar_family)
        .map_err(|_| ApplicationOperationBodyRequirementDenial::UnsupportedScalar)?;
    Ok(if presence == ApplicationFieldPresence::Optional {
        ExpressionType::option(ty)
    } else {
        ty
    })
}
