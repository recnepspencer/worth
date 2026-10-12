mod admitted_entity_field_lookup;
mod bounded_entity_field_lookup;
mod pinned_entity_field_lookup;
pub use bounded_entity_field_lookup::BoundedEntityFieldLookupRead;
mod selected_entity_field_lookup;
pub use selected_entity_field_lookup::BoundedEntityFieldSelectedLookup;
mod bounded_related_entity_ordered_lookup;
mod bounded_relation_join_lookup;
mod definition_lookup;
mod entity_field_collection;
#[cfg(test)]
mod execution;
mod prepared_entity_field_lookup;
mod prepared_field_certification;
#[cfg(test)]
mod routing;
#[cfg(test)]
mod scratch;

mod pinning;
#[cfg(test)]
pub(crate) use self::scratch::{index_query_scratch_hint_exists, purge_index_query_scratch_hints};
use crate::runtime::RelationalRuntime;
pub use definition_lookup::DerivedIndexDefinitionLookup;
pub use pinning::IndexAccess;
pub use prepared_entity_field_lookup::PreparedEntityFieldLookup;
impl RelationalRuntime {
    pub fn index_access(&self) -> IndexAccess<'_> {
        IndexAccess::new(self)
    }
}
