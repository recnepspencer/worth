use worth_foundational::facade::{AspectFieldLocator, FieldKey};

use crate::identity::data::KindId;
use crate::schema::data::{LoweredAspectContractBinding, LoweredAspectContractPlan};

#[derive(Debug, Clone)]
pub struct EntityIndexFieldProjectionScope {
    kind_id: KindId,
}

#[derive(Debug, Clone)]
pub struct RelationIndexFieldProjectionScope {
    kind_id: KindId,
}

impl EntityIndexFieldProjectionScope {
    pub const fn kind_id(&self) -> KindId {
        self.kind_id
    }
}

impl RelationIndexFieldProjectionScope {
    pub const fn kind_id(&self) -> KindId {
        self.kind_id
    }
}

pub fn entity_index_projection_scope(
    plan: &LoweredAspectContractPlan,
    field_locator: &AspectFieldLocator,
) -> Option<EntityIndexFieldProjectionScope> {
    let field = single_field_locator_key(field_locator)?;
    let binding = matching_binding(plan, field_locator)?;
    if binding.targets_entity_scalar_field(field) || binding.targets_entity_struct_field(field) {
        return Some(EntityIndexFieldProjectionScope {
            kind_id: plan.kind_id,
        });
    }
    None
}

pub fn relation_index_projection_scope(
    plan: &LoweredAspectContractPlan,
    field_locator: &AspectFieldLocator,
) -> Option<RelationIndexFieldProjectionScope> {
    let field = single_field_locator_key(field_locator)?;
    let binding = matching_binding(plan, field_locator)?;
    if binding.targets_relation_scalar_field(field) || binding.targets_relation_struct_field(field)
    {
        return Some(RelationIndexFieldProjectionScope {
            kind_id: plan.kind_id,
        });
    }
    None
}

fn matching_binding<'a>(
    plan: &'a LoweredAspectContractPlan,
    field_locator: &AspectFieldLocator,
) -> Option<&'a LoweredAspectContractBinding> {
    plan.executable_bindings
        .iter()
        .find(|binding| binding.aspect_key() == field_locator.aspect().aspect_key())
}

fn single_field_locator_key(field_locator: &AspectFieldLocator) -> Option<&FieldKey> {
    match field_locator.field_path().fields() {
        [field] => Some(field),
        _ => None,
    }
}
