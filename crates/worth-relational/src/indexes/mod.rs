mod retained_definition;
pub use retained_definition::RetainedDerivedIndexDefinition;
pub(crate) use retained_definition::{IndexDefinitionBacking, RetainedIndexDefinitionMeaning};
mod access;
pub use access::{
    BoundedEntityFieldLookupRead, BoundedEntityFieldSelectedLookup, DerivedIndexDefinitionLookup,
    PreparedEntityFieldLookup,
};
mod authority;
#[cfg(test)]
pub(crate) use authority::at_next_charges;
pub(crate) use authority::PreparedCandidateIndexPublication;
pub(crate) use authority::{persistent_map_path_bytes, row_copy_bounds};
pub mod data;
mod projected_field_values;
mod unique_entity_aspect_field_index;

#[cfg(test)]
pub(crate) use access::index_query_scratch_hint_exists;
#[cfg(test)]
pub(crate) use access::purge_index_query_scratch_hints;

#[cfg(test)]
pub(crate) use authority::last_allocation_bytes;
