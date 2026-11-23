//! Named operand slots a where filter binds: row fields and query parameters.
use super::ApplicationQueryWhereFilterDenial as Denial;
use crate::application_schema::ApplicationFieldPresence;
use worth_foundational::{expression_api::ExpressionType, facade::ScalarAspectType};

/// The authority-free source of one operand. Installation binds the field to
/// its admitted locator; admission binds the parameter to its request value.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryWhereFilterSource {
    /// A declared field of the query's root entity, read once per candidate.
    /// An optional field is an `Option` operand; absence is `none`, never a
    /// sentinel value.
    Field {
        entity: String,
        aspect: String,
        field: String,
        scalar_family: ScalarAspectType,
        presence: ApplicationFieldPresence,
    },
    /// A typed query parameter, bound once per request.
    Parameter {
        parameter: String,
        scalar_family: ScalarAspectType,
    },
    /// A request parameter whose value or absence is supplied explicitly.
    OptionalParameter {
        parameter: String,
        scalar_family: ScalarAspectType,
    },
}

impl ApplicationQueryWhereFilterSource {
    pub const fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Field { scalar_family, .. }
            | Self::Parameter { scalar_family, .. }
            | Self::OptionalParameter { scalar_family, .. } => *scalar_family,
        }
    }

    /// The exact expression type this source contributes to the operand
    /// schema.
    pub fn expression_type(&self) -> Result<ExpressionType, Denial> {
        let scalar = ExpressionType::from_scalar_family(self.scalar_family())
            .map_err(|_| Denial::UnsupportedScalar)?;
        Ok(match self {
            Self::Field {
                presence: ApplicationFieldPresence::Optional,
                ..
            }
            | Self::OptionalParameter { .. } => ExpressionType::option(scalar),
            Self::Field { .. } | Self::Parameter { .. } => scalar,
        })
    }

    pub(super) const fn kind_name(&self) -> &'static str {
        match self {
            Self::Field { .. } => "field",
            Self::Parameter { .. } => "parameter",
            Self::OptionalParameter { .. } => "optional-parameter",
        }
    }
}

/// One named slot of the filter's Foundation operand schema.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryWhereFilterOperand {
    name: String,
    source: ApplicationQueryWhereFilterSource,
}

impl ApplicationQueryWhereFilterOperand {
    pub fn from_untrusted(name: String, source: ApplicationQueryWhereFilterSource) -> Self {
        Self { name, source }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn source(&self) -> &ApplicationQueryWhereFilterSource {
        &self.source
    }

    /// `(entity, aspect, field)` when this operand reads a row field.
    pub fn field(&self) -> Option<(&str, &str, &str)> {
        match &self.source {
            ApplicationQueryWhereFilterSource::Field {
                entity,
                aspect,
                field,
                ..
            } => Some((entity, aspect, field)),
            ApplicationQueryWhereFilterSource::Parameter { .. }
            | ApplicationQueryWhereFilterSource::OptionalParameter { .. } => None,
        }
    }

    /// The parameter name when this operand reads a query parameter.
    pub fn parameter(&self) -> Option<&str> {
        match &self.source {
            ApplicationQueryWhereFilterSource::Parameter { parameter, .. }
            | ApplicationQueryWhereFilterSource::OptionalParameter { parameter, .. } => {
                Some(parameter)
            }
            ApplicationQueryWhereFilterSource::Field { .. } => None,
        }
    }

    /// Stable `(key, value)` pairs for canonical identity material.
    pub(in crate::application_query) fn canonical_parts(&self) -> Vec<(&'static str, &str)> {
        let mut parts = vec![("name", self.name()), ("kind", self.source.kind_name())];
        match &self.source {
            ApplicationQueryWhereFilterSource::Field {
                entity,
                aspect,
                field,
                scalar_family,
                presence,
            } => parts.extend([
                ("entity", entity.as_str()),
                ("aspect", aspect.as_str()),
                ("field", field.as_str()),
                ("scalar", scalar_family.canonical_name()),
                ("presence", presence.canonical_name()),
            ]),
            ApplicationQueryWhereFilterSource::Parameter {
                parameter,
                scalar_family,
            }
            | ApplicationQueryWhereFilterSource::OptionalParameter {
                parameter,
                scalar_family,
            } => parts.extend([
                ("parameter", parameter.as_str()),
                ("scalar", scalar_family.canonical_name()),
            ]),
        }
        parts
    }
}
