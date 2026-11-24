pub(crate) mod applied_entity_field_lookup;
pub(crate) mod bounded_entity_field_lookup;
mod field_candidate_comparison;
pub use applied_entity_field_lookup::{
    AppliedEntityFieldLookupDenial, AppliedEntityFieldLookupOutcome, AppliedEntityFieldMatch,
};
pub(crate) mod pinned_entity_field_lookup;
pub use pinning::BoundedEntityFieldLookupRead;
mod selected_entity_field_lookup;
pub use selected_entity_field_lookup::BoundedEntityFieldSelectedLookup;
pub(crate) mod bounded_related_entity_ordered_lookup;
pub(crate) mod bounded_relation_join_lookup;
mod definition_lookup;
mod entity_field_collection;
pub(crate) mod prepared_entity_field_lookup;
mod prepared_field_certification;
mod prepared_lookup_admission;
pub(crate) mod related_entity_prefix;

mod pinning;
use crate::runtime::RelationalRuntime;
pub use definition_lookup::DerivedIndexDefinitionLookup;
pub use pinning::IndexAccess;
pub use prepared_entity_field_lookup::PreparedEntityFieldLookup;
impl RelationalRuntime {
    pub fn index_access(&self) -> IndexAccess<'_> {
        IndexAccess::new(self)
    }
}
pub(crate) mod selected_index;
