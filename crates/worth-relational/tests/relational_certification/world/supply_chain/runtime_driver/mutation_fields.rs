use std::collections::BTreeMap;

use crate::relational_certification::world::supply_chain::{
    EntityKey, RelationKey, SupplyChainSemanticHandles,
};
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, UpdateEntityFieldsIntent,
    UpdateRelationEndpointsIntent,
};

pub(super) fn update_fields<const N: usize>(
    handles: &SupplyChainSemanticHandles,
    entity: EntityKey,
    fields: [(SupplyChainField, AspectValue); N],
) -> MutationIntent {
    let fields = fields
        .into_iter()
        .map(|(field, value)| {
            let name = field.canonical_name();
            (
                planned_single_field_locator(
                    AspectKey::new(name).expect("canonical Supply Chain aspect"),
                    FieldKey::new(name).expect("canonical Supply Chain field"),
                ),
                value,
            )
        })
        .collect::<BTreeMap<_, _>>();
    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
        UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: AspectFieldPatch::new(fields),
        },
    ))
}

pub(super) fn update_relation(
    handles: &SupplyChainSemanticHandles,
    relation: RelationKey,
    source: EntityKey,
    target: EntityKey,
) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id: handles.relations[&relation].id,
            kind_id: crate::relational_certification::world::supply_chain::relation_kind_id(
                relation.kind,
            ),
            source: EntityReference::Existing(handles.entities[&source].id),
            target: EntityReference::Existing(handles.entities[&target].id),
        },
    ))
}

pub(super) fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}

pub(super) fn number(value: impl Into<u64>) -> AspectValue {
    AspectValue::UInt64(value.into())
}

#[derive(Clone, Copy)]
pub(super) enum SupplyChainField {
    Arrival,
    Booking,
    Capacity,
    Hazard,
    Posture,
    Result,
    Revision,
    Status,
}

impl SupplyChainField {
    const fn canonical_name(self) -> &'static str {
        match self {
            Self::Arrival => "arrival",
            Self::Booking => "booking",
            Self::Capacity => "capacity",
            Self::Hazard => "hazard",
            Self::Posture => "posture",
            Self::Result => "result",
            Self::Revision => "revision",
            Self::Status => "status",
        }
    }
}
