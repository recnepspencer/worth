use super::{AspectKey, FieldKey, ScalarAspectType, WorthQueryInstalledGraphPredicate};
use worth_query_declaration::facade::application_query::ApplicationQueryRelationPredicate;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorthQueryInstalledGraphRelationPredicate {
    EqualParameter(WorthQueryInstalledGraphPredicate),
    FieldAbsent {
        entity: String,
        aspect: AspectKey,
        field: FieldKey,
        scalar_family: ScalarAspectType,
    },
}

impl WorthQueryInstalledGraphRelationPredicate {
    pub(super) fn compile(predicate: &ApplicationQueryRelationPredicate) -> Self {
        let (entity, aspect, field) = predicate.field();
        let aspect = super::compilation::admitted_aspect_key(aspect);
        let field = super::compilation::admitted_field_key(field);
        match predicate {
            ApplicationQueryRelationPredicate::EqualParameter(equal) => {
                Self::EqualParameter(WorthQueryInstalledGraphPredicate {
                    entity: entity.to_owned(),
                    aspect,
                    field,
                    parameter: equal.parameter().to_owned(),
                    scalar_family: equal.scalar_family(),
                })
            }
            ApplicationQueryRelationPredicate::FieldAbsent { scalar_family, .. } => {
                Self::FieldAbsent {
                    entity: entity.to_owned(),
                    aspect,
                    field,
                    scalar_family: *scalar_family,
                }
            }
        }
    }

    pub fn field(&self) -> (&str, &str, &str) {
        match self {
            Self::EqualParameter(equal) => equal.field(),
            Self::FieldAbsent {
                entity,
                aspect,
                field,
                ..
            } => (entity, aspect.as_str(), field.as_str()),
        }
    }

    pub fn aspect_key(&self) -> &AspectKey {
        match self {
            Self::EqualParameter(equal) => equal.aspect_key(),
            Self::FieldAbsent { aspect, .. } => aspect,
        }
    }

    pub fn field_key(&self) -> &FieldKey {
        match self {
            Self::EqualParameter(equal) => equal.field_key(),
            Self::FieldAbsent { field, .. } => field,
        }
    }

    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::EqualParameter(equal) => Some(equal.parameter()),
            Self::FieldAbsent { .. } => None,
        }
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::EqualParameter(equal) => equal.scalar_family(),
            Self::FieldAbsent { scalar_family, .. } => *scalar_family,
        }
    }
}
