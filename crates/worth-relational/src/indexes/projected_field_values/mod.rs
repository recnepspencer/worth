mod checked_entry_map;
mod entity_field_index_values;
mod exact_entity_field_match;
pub(crate) use worth_relational_contracts::internal_port::indexes::projected_field_values::field_projection_scope;
mod index_projection_source;
#[cfg(test)]
pub(crate) use index_projection_source::verify_native_field_navigation;
mod related_entity_ordering;
mod relation_field_index_values;
mod relation_join;
pub(in crate::indexes) use field_projection_scope::{
    entity_index_projection_scope, relation_index_projection_scope,
};

pub(super) use checked_entry_map::checked_finish;
pub(super) use entity_field_index_values::{
    build_entity_aspect_field_index, build_entity_aspect_field_index_checked,
    entity_aspect_field_ordering_value,
};
pub(super) use exact_entity_field_match::exact_entity_field_matches;
pub(crate) use index_projection_source::IndexProjectionSource;
pub(super) use related_entity_ordering::{
    build_related_entity_ordering_index, build_related_entity_ordering_index_checked,
    compare_ordering_value, compare_related_entries, RelatedEntityOrderingProjection,
};
pub(super) use relation_field_index_values::{
    build_relation_aspect_field_index, build_relation_aspect_field_index_checked,
};
pub(super) use relation_join::{
    build_relation_join_index, build_relation_join_index_checked, join_endpoints,
};

mod scoped_parity;
pub(super) use scoped_parity::{joined_rows_for_pair, ordered_children_for_parent};

mod selected_projection_scopes;
