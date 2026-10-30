pub use std::collections::BTreeMap;

pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::RelationKey,
};
pub use {
    worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue,
    worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString,
};
pub use {
    worth_relational::facade::transactions::planned_single_field_locator,
    worth_relational::facade::transactions::AspectFieldPatch,
    worth_relational::facade::transactions::EntityMutationIntent,
    worth_relational::facade::transactions::EntityReference,
    worth_relational::facade::transactions::MutationIntent,
    worth_relational::facade::transactions::RelationMutationIntent,
    worth_relational::facade::transactions::UpdateEntityFieldsIntent,
    worth_relational::facade::transactions::UpdateRelationEndpointsIntent,
};

pub fn update_fields<const N: usize>(
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
                    worth_foundational::facade::AspectKey::new(name)
                        .expect("canonical Supply Chain aspect"),
                    worth_foundational::facade::FieldKey::new(name)
                        .expect("canonical Supply Chain field"),
                ),
                value,
            )
        })
        .collect::<BTreeMap<_, _>>();
    worth_relational::facade::transactions::MutationIntent::Entity(
        worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: handles.entities[&entity].id,
                fields: worth_relational::facade::transactions::AspectFieldPatch::new(fields),
            },
        ),
    )
}

pub fn update_relation(
    handles: &SupplyChainSemanticHandles,
    relation: RelationKey,
    source: EntityKey,
    target: EntityKey,
) -> MutationIntent {
    worth_relational::facade::transactions::MutationIntent::Relation(worth_relational::facade::transactions::RelationMutationIntent::UpdateEndpoints(
        UpdateRelationEndpointsIntent {
            relation_id: handles.relations[&relation].id,
            kind_id: crate::relational_certification::ordinary_context::world::supply_chain::schema_vocabulary::relation_kind_id(
                relation.kind,
            ),
            source: worth_relational::facade::transactions::EntityReference::Existing(handles.entities[&source].id),
            target: worth_relational::facade::transactions::EntityReference::Existing(handles.entities[&target].id),
        },
    ))
}

pub fn text(value: &str) -> AspectValue {
    worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Raw(value.to_owned()),
    )
}

pub fn number(value: impl Into<u64>) -> AspectValue {
    worth_foundational::facade::AspectValue::UInt64(value.into())
}

#[derive(Clone, Copy)]
pub enum SupplyChainField {
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
    pub const fn canonical_name(self) -> &'static str {
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
