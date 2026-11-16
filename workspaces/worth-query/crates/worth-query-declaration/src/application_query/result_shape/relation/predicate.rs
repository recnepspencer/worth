use worth_foundational::facade::ScalarAspectType;

use crate::application_query::ApplicationQueryPredicate;

/// The complete row-selection meaning of one declared result relation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryRelationPredicate {
    EqualParameter(ApplicationQueryPredicate),
    FieldAbsent {
        entity: String,
        aspect: String,
        field: String,
        scalar_family: ScalarAspectType,
    },
    EntityEqualParameter {
        entity: String,
        parameter: String,
    },
}

impl ApplicationQueryRelationPredicate {
    pub fn field(&self) -> Option<(&str, &str, &str)> {
        match self {
            Self::EqualParameter(predicate) => Some(predicate.field()),
            Self::FieldAbsent {
                entity,
                aspect,
                field,
                ..
            } => Some((entity, aspect, field)),
            Self::EntityEqualParameter { .. } => None,
        }
    }

    pub fn entity(&self) -> &str {
        match self {
            Self::EqualParameter(predicate) => predicate.field().0,
            Self::FieldAbsent { entity, .. } | Self::EntityEqualParameter { entity, .. } => entity,
        }
    }

    pub const fn operator(&self) -> &'static str {
        match self {
            Self::EqualParameter(_) => "equal-parameter",
            Self::FieldAbsent { .. } => "field-absent",
            Self::EntityEqualParameter { .. } => "entity-equal-parameter",
        }
    }

    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::EqualParameter(predicate) => Some(predicate.parameter()),
            Self::FieldAbsent { .. } => None,
            Self::EntityEqualParameter { parameter, .. } => Some(parameter),
        }
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::EqualParameter(predicate) => predicate.scalar_family(),
            Self::FieldAbsent { scalar_family, .. } => *scalar_family,
            Self::EntityEqualParameter { .. } => ScalarAspectType::EntityRef,
        }
    }
}
