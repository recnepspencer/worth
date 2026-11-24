//! Named operand slots a where filter binds: row fields, query parameters,
//! and authored literal constants.
use super::{ApplicationQueryWhereFilterDenial as Denial, ApplicationQueryWhereFilterProgram};
use crate::application_schema::ApplicationFieldPresence;
use std::borrow::Cow;
use worth_foundational::{
    expression_api::{ExpressionResource, ExpressionType, ExpressionValue},
    facade::{prepare_aspect_value_identity_basis, AspectValue, ScalarAspectType},
};

/// The authority-free source of one operand. Installation binds fields to
/// admitted locators, admission binds parameters to request values, and
/// literals carry their authored constant directly.
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
    /// A typed authored scalar constant, bound once per query.
    Literal {
        scalar_family: ScalarAspectType,
        value: AspectValue,
    },
}

impl ApplicationQueryWhereFilterSource {
    pub const fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Field { scalar_family, .. }
            | Self::Parameter { scalar_family, .. }
            | Self::OptionalParameter { scalar_family, .. }
            | Self::Literal { scalar_family, .. } => *scalar_family,
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
            Self::Field { .. } | Self::Parameter { .. } | Self::Literal { .. } => scalar,
        })
    }

    /// Verifies that a literal source carries a bounded, lossless value of its
    /// declared scalar family.
    pub(super) fn validate_value(&self) -> Result<(), Denial> {
        let Self::Literal {
            scalar_family,
            value,
        } = self
        else {
            return Ok(());
        };
        let limit = ApplicationQueryWhereFilterProgram::PROFILE
            .limit(ExpressionResource::SourceBytes);
        if value.value_family() != *scalar_family
            || (value.semantic_byte_width() as u64) > limit
            || ExpressionValue::from_aspect_value(value).is_err()
        {
            return Err(Denial::InvalidLiteralValue);
        }
        Ok(())
    }

    pub(super) const fn kind_name(&self) -> &'static str {
        match self {
            Self::Field { .. } => "field",
            Self::Parameter { .. } => "parameter",
            Self::OptionalParameter { .. } => "optional-parameter",
            Self::Literal { .. } => "literal",
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
            | ApplicationQueryWhereFilterSource::OptionalParameter { .. }
            | ApplicationQueryWhereFilterSource::Literal { .. } => None,
        }
    }

    /// The parameter name when this operand reads a query parameter.
    pub fn parameter(&self) -> Option<&str> {
        match &self.source {
            ApplicationQueryWhereFilterSource::Parameter { parameter, .. }
            | ApplicationQueryWhereFilterSource::OptionalParameter { parameter, .. } => {
                Some(parameter)
            }
            ApplicationQueryWhereFilterSource::Field { .. }
            | ApplicationQueryWhereFilterSource::Literal { .. } => None,
        }
    }

    /// The authored constant when this operand is a literal value.
    pub const fn literal_value(&self) -> Option<&AspectValue> {
        match &self.source {
            ApplicationQueryWhereFilterSource::Literal { value, .. } => Some(value),
            ApplicationQueryWhereFilterSource::Field { .. }
            | ApplicationQueryWhereFilterSource::Parameter { .. }
            | ApplicationQueryWhereFilterSource::OptionalParameter { .. } => None,
        }
    }

    /// Stable `(key, value)` pairs for canonical identity material.
    pub(in crate::application_query) fn canonical_parts(&self) -> Vec<(&'static str, Cow<'_, str>)> {
        let mut parts = vec![
            ("name", Cow::Borrowed(self.name())),
            ("kind", Cow::Borrowed(self.source.kind_name())),
        ];
        match &self.source {
            ApplicationQueryWhereFilterSource::Field {
                entity,
                aspect,
                field,
                scalar_family,
                presence,
            } => parts.extend([
                ("entity", Cow::Borrowed(entity.as_str())),
                ("aspect", Cow::Borrowed(aspect.as_str())),
                ("field", Cow::Borrowed(field.as_str())),
                ("scalar", Cow::Borrowed(scalar_family.canonical_name())),
                ("presence", Cow::Borrowed(presence.canonical_name())),
            ]),
            ApplicationQueryWhereFilterSource::Parameter {
                parameter,
                scalar_family,
            }
            | ApplicationQueryWhereFilterSource::OptionalParameter {
                parameter,
                scalar_family,
            } => parts.extend([
                ("parameter", Cow::Borrowed(parameter.as_str())),
                ("scalar", Cow::Borrowed(scalar_family.canonical_name())),
            ]),
            ApplicationQueryWhereFilterSource::Literal {
                scalar_family,
                value,
            } => parts.extend([
                ("scalar", Cow::Borrowed(scalar_family.canonical_name())),
                (
                    "value",
                    Cow::Owned(prepare_aspect_value_identity_basis(value).as_str().to_owned()),
                ),
            ]),
        }
        parts
    }
}
