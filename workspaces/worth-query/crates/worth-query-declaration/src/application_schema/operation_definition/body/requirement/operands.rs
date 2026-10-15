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
    /// True when at least one current native attestation matches this exact producer.
    SignedEvidence(crate::application_schema::ApplicationSigningEvidenceSource),
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
            Self::SignedEvidence(source) => &source.lifecycle.source_entity,
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
            Self::SignedEvidence(source) => source.decision_reads(),
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
        use ScalarAspectType as Scalar;
        let (scalar_family, presence, presence_only) = match self {
            Self::SignedEvidence(_) => return Ok(ExpressionType::Bool),
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
        let ty = match scalar_family {
            Scalar::Bool => ExpressionType::Bool,
            Scalar::Int8 => ExpressionType::Integer(IntegerType::Int8),
            Scalar::Int16 => ExpressionType::Integer(IntegerType::Int16),
            Scalar::Int32 => ExpressionType::Integer(IntegerType::Int32),
            Scalar::Int64 => ExpressionType::Integer(IntegerType::Int64),
            Scalar::UInt8 => ExpressionType::Integer(IntegerType::UInt8),
            Scalar::UInt16 => ExpressionType::Integer(IntegerType::UInt16),
            Scalar::UInt32 => ExpressionType::Integer(IntegerType::UInt32),
            Scalar::UInt64 => ExpressionType::Integer(IntegerType::UInt64),
            Scalar::Float32 => ExpressionType::Float32,
            Scalar::Float64 => ExpressionType::Float64,
            Scalar::Decimal => ExpressionType::Decimal,
            Scalar::String => ExpressionType::String,
            _ => return Err(ApplicationOperationBodyRequirementDenial::UnsupportedScalar),
        };
        Ok(if presence == ApplicationFieldPresence::Optional {
            ExpressionType::option(ty)
        } else {
            ty
        })
    }
}
