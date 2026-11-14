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
}

impl ApplicationQueryRelationPredicate {
    pub fn field(&self) -> (&str, &str, &str) {
        match self {
            Self::EqualParameter(predicate) => predicate.field(),
            Self::FieldAbsent {
                entity,
                aspect,
                field,
                ..
            } => (entity, aspect, field),
        }
    }

    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::EqualParameter(predicate) => Some(predicate.parameter()),
            Self::FieldAbsent { .. } => None,
        }
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::EqualParameter(predicate) => predicate.scalar_family(),
            Self::FieldAbsent { scalar_family, .. } => *scalar_family,
        }
    }
}
