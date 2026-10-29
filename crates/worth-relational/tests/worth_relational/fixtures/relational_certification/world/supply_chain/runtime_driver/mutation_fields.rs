pub(crate) use std::collections::BTreeMap;

pub(crate) use crate::fast::relational_certification::world::supply_chain::{
    EntityKey, RelationKey, SupplyChainSemanticHandles,
};
pub(crate) use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
pub(crate) use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, EntityReference,
    MutationIntent, RelationMutationIntent, UpdateEntityFieldsIntent,
    UpdateRelationEndpointsIntent,
};

pub(crate) fn update_fields<const N: usize>(
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

pub(crate) fn update_relation(
    handles: &SupplyChainSemanticHandles,
    relation: RelationKey,
    source: EntityKey,
    target: EntityKey,
) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id: handles.relations[&relation].id,
            kind_id: crate::fast::relational_certification::world::supply_chain::relation_kind_id(
                relation.kind,
            ),
            source: EntityReference::Existing(handles.entities[&source].id),
            target: EntityReference::Existing(handles.entities[&target].id),
        },
    ))
}

pub(crate) fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.to_owned()))
}

pub(crate) fn number(value: impl Into<u64>) -> AspectValue {
    AspectValue::UInt64(value.into())
}

#[derive(Clone, Copy)]
pub(crate) enum SupplyChainField {
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
