mod access;
pub use access::DerivedIndexDefinitionLookup;
mod authority;
pub(crate) use authority::PreparedCandidateIndexPublication;
pub mod data;
mod projected_field_values;
mod unique_entity_aspect_field_index;

#[cfg(test)]
pub(crate) use access::index_query_scratch_hint_exists;
pub(crate) use access::purge_index_query_scratch_hints;
