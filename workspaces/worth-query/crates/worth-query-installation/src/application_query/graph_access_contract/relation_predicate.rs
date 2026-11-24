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
    EntityEqualParameter {
        entity: String,
        parameter: String,
    },
}

impl WorthQueryInstalledGraphRelationPredicate {
    pub(super) fn compile(predicate: &ApplicationQueryRelationPredicate) -> Self {
        match predicate {
            ApplicationQueryRelationPredicate::EqualParameter(equal) => {
                let (entity, aspect, field) = equal.field();
                Self::EqualParameter(WorthQueryInstalledGraphPredicate {
                    entity: entity.to_owned(),
                    aspect: super::compilation::admitted_aspect_key(aspect),
                    field: super::compilation::admitted_field_key(field),
                    expected: equal.expected().clone(),
                    scalar_family: equal.scalar_family(),
                })
            }
            ApplicationQueryRelationPredicate::FieldAbsent {
                entity,
                aspect,
                field,
                scalar_family,
            } => Self::FieldAbsent {
                entity: entity.clone(),
                aspect: super::compilation::admitted_aspect_key(aspect),
                field: super::compilation::admitted_field_key(field),
                scalar_family: *scalar_family,
            },
            ApplicationQueryRelationPredicate::EntityEqualParameter { entity, parameter } => {
                Self::EntityEqualParameter {
                    entity: entity.clone(),
                    parameter: parameter.clone(),
                }
            }
        }
    }

    pub fn entity(&self) -> &str {
        match self {
            Self::EqualParameter(equal) => equal.field().0,
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

    pub fn field(&self) -> Option<(&str, &str, &str)> {
        self.field_keys()
            .map(|(aspect, field)| (self.entity(), aspect.as_str(), field.as_str()))
    }

    pub fn field_keys(&self) -> Option<(&AspectKey, &FieldKey)> {
        match self {
            Self::EqualParameter(equal) => Some((equal.aspect_key(), equal.field_key())),
            Self::FieldAbsent { aspect, field, .. } => Some((aspect, field)),
            Self::EntityEqualParameter { .. } => None,
        }
    }

    pub fn parameter(&self) -> Option<&str> {
        match self {
            Self::EqualParameter(equal) => equal.parameter(),
            Self::FieldAbsent { .. } => None,
            Self::EntityEqualParameter { parameter, .. } => Some(parameter),
        }
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::EqualParameter(equal) => equal.scalar_family(),
            Self::FieldAbsent { scalar_family, .. } => *scalar_family,
            Self::EntityEqualParameter { .. } => ScalarAspectType::EntityRef,
        }
    }
}
