use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{
    AspectMask, CanonicalFieldPath, DiagnosticMask, FieldKey, ProjectionMask, ScalarAspectType,
};

use crate::application_schema::ApplicationFieldPresence;

use super::super::{
    result_slot_key::{
        ApplicationQueryResultFieldSlotContract, ApplicationQueryResultRelationSlotContract,
    },
    ApplicationQueryCardinality, ApplicationQueryResultSlotKey,
    ApplicationQueryResultTraversalDirection,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryDisclosureSelector {
    Orbit(super::super::ApplicationQueryOrbitMeaning),
    InternalField {
        entity: String,
        aspect: String,
        field: String,
        projection_mask: AspectMask<ProjectionMask>,
        diagnostic_mask: AspectMask<DiagnosticMask>,
    },
    Field {
        query_type: WorthQueryPortableTypeIdentity,
        slot_type: WorthQueryPortableTypeIdentity,
        entity: String,
        aspect: String,
        field: String,
        output_name: String,
        scalar_family: ScalarAspectType,
        value_type: WorthQueryPortableTypeIdentity,
        presence: ApplicationFieldPresence,
        projection_mask: AspectMask<ProjectionMask>,
        diagnostic_mask: AspectMask<DiagnosticMask>,
    },
    Relation {
        query_type: WorthQueryPortableTypeIdentity,
        slot_type: WorthQueryPortableTypeIdentity,
        relation: String,
        from: String,
        to: String,
        direction: ApplicationQueryResultTraversalDirection,
        cardinality: ApplicationQueryCardinality,
        output_name: String,
    },
}

impl ApplicationQueryDisclosureSelector {
    pub(crate) fn portable_identities_are_valid(&self) -> bool {
        match self {
            Self::InternalField { .. } => true,
            Self::Orbit(orbit) => {
                orbit.query_identity().is_valid() && orbit.slot_identity().is_valid()
            }
            Self::Field {
                query_type,
                slot_type,
                value_type,
                ..
            } => query_type.is_valid() && slot_type.is_valid() && value_type.is_valid(),
            Self::Relation {
                query_type,
                slot_type,
                ..
            } => query_type.is_valid() && slot_type.is_valid(),
        }
    }

    pub(crate) fn has_exact_field_masks(&self) -> bool {
        let Some((_, _, field)) = self.field_contract() else {
            return true;
        };
        let Some(field) = FieldKey::new(field) else {
            return false;
        };
        let expected = CanonicalFieldPath::single(field);
        mask_is_exact(self.projection_mask(), &expected)
            && mask_is_exact(self.diagnostic_mask(), &expected)
    }

    pub fn result_slot_key(&self) -> Option<ApplicationQueryResultSlotKey> {
        match self {
            Self::Orbit(orbit) => Some(orbit.slot_key()),
            Self::Field {
                query_type,
                slot_type,
                entity,
                aspect,
                field,
                output_name,
                scalar_family,
                value_type,
                presence,
                ..
            } => Some(ApplicationQueryResultSlotKey::field(
                query_type.clone(),
                slot_type.clone(),
                ApplicationQueryResultFieldSlotContract {
                    entity,
                    aspect,
                    field,
                    output_name,
                    scalar_family: *scalar_family,
                    value_type: value_type.clone(),
                    presence: *presence,
                },
            )),
            Self::Relation {
                query_type,
                slot_type,
                relation,
                from,
                to,
                direction,
                output_name,
                cardinality,
            } => Some(ApplicationQueryResultSlotKey::relation(
                query_type.clone(),
                slot_type.clone(),
                ApplicationQueryResultRelationSlotContract {
                    relation,
                    from,
                    to,
                    direction: *direction,
                    output_name,
                    cardinality: *cardinality,
                },
            )),
            Self::InternalField { .. } => None,
        }
    }

    pub const fn is_internal_computation(&self) -> bool {
        matches!(self, Self::InternalField { .. })
    }

    pub const fn slot_type(&self) -> &str {
        match self {
            Self::Orbit(orbit) => orbit.slot_type(),
            Self::Field { slot_type, .. } | Self::Relation { slot_type, .. } => slot_type.as_str(),
            Self::InternalField { .. } => "internal-computation",
        }
    }

    pub const fn query_type(&self) -> &str {
        match self {
            Self::Orbit(orbit) => orbit.query_type(),
            Self::Field { query_type, .. } | Self::Relation { query_type, .. } => {
                query_type.as_str()
            }
            Self::InternalField { .. } => "internal-computation",
        }
    }

    pub fn output_name(&self) -> &str {
        match self {
            Self::Orbit(orbit) => orbit.output_name(),
            Self::Field { output_name, .. } | Self::Relation { output_name, .. } => output_name,
            Self::InternalField { .. } => "internal-computation",
        }
    }

    pub fn field_contract(&self) -> Option<(&str, &str, &str)> {
        match self {
            Self::InternalField {
                entity,
                aspect,
                field,
                ..
            }
            | Self::Field {
                entity,
                aspect,
                field,
                ..
            } => Some((entity, aspect, field)),
            Self::Relation { .. } | Self::Orbit(_) => None,
        }
    }

    pub fn relation_contract(
        &self,
    ) -> Option<(
        &str,
        &str,
        &str,
        ApplicationQueryResultTraversalDirection,
        ApplicationQueryCardinality,
    )> {
        match self {
            Self::Relation {
                relation,
                from,
                to,
                direction,
                cardinality,
                ..
            } => Some((relation, from, to, *direction, *cardinality)),
            Self::Field { .. } | Self::InternalField { .. } | Self::Orbit(_) => None,
        }
    }

    pub const fn projection_mask(&self) -> Option<&AspectMask<ProjectionMask>> {
        match self {
            Self::InternalField {
                projection_mask, ..
            }
            | Self::Field {
                projection_mask, ..
            } => Some(projection_mask),
            Self::Relation { .. } | Self::Orbit(_) => None,
        }
    }

    pub const fn diagnostic_mask(&self) -> Option<&AspectMask<DiagnosticMask>> {
        match self {
            Self::InternalField {
                diagnostic_mask, ..
            }
            | Self::Field {
                diagnostic_mask, ..
            } => Some(diagnostic_mask),
            Self::Relation { .. } | Self::Orbit(_) => None,
        }
    }
}

fn mask_is_exact<Mode>(mask: Option<&AspectMask<Mode>>, expected: &CanonicalFieldPath) -> bool {
    mask.is_some_and(|mask| mask.paths() == std::slice::from_ref(expected))
}
