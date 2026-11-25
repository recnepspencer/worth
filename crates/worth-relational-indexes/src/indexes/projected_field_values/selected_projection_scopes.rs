//! Projects field scope from the selected schema authority.
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::indexes::projected_field_values::field_projection_scope::{
    entity_index_projection_scope, relation_index_projection_scope,
    EntityIndexFieldProjectionScope, RelationIndexFieldProjectionScope,
};
use worth_foundational::facade::AspectFieldLocator;

pub(super) fn entity_index_projection_scopes(
    source: &super::IndexProjectionSource<'_>,
    field_locator: &AspectFieldLocator,
) -> Vec<EntityIndexFieldProjectionScope> {
    source
        .aspect_plans()
        .into_iter()
        .flat_map(|catalog| catalog.entity_plans.values())
        .filter_map(|plan| entity_index_projection_scope(plan, field_locator))
        .collect()
}

pub(super) fn source_entity_index_projection_scope_for_kind(
    source: &super::IndexProjectionSource<'_>,
    kind_id: KindId,
    field_locator: &AspectFieldLocator,
) -> Option<EntityIndexFieldProjectionScope> {
    let plan = source.entity_aspect_plan(kind_id)?;
    entity_index_projection_scope(plan, field_locator)
}

pub(super) fn relation_index_projection_scopes(
    source: &super::IndexProjectionSource<'_>,
    field_locator: &AspectFieldLocator,
) -> Vec<RelationIndexFieldProjectionScope> {
    source
        .aspect_plans()
        .into_iter()
        .flat_map(|catalog| catalog.relation_plans.values())
        .filter_map(|plan| relation_index_projection_scope(plan, field_locator))
        .collect()
}
