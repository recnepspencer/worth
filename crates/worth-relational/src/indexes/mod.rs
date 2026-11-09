pub(crate) use authority::{
    checkpoint_index_projections, verify_checkpoint_index_bases, CheckpointIndexProjections,
};
mod access;
pub use access::{
    AppliedEntityFieldLookupDenial, AppliedEntityFieldLookupOutcome, AppliedEntityFieldMatch,
    BoundedEntityFieldLookupRead, BoundedEntityFieldSelectedLookup, DerivedIndexDefinitionLookup,
    PreparedEntityFieldLookup,
};
mod authority;
#[cfg(test)]
pub(crate) use authority::at_next_charges;
pub(crate) use authority::AppliedFieldImage;
pub(crate) use authority::{PreparedCandidateIndexPublication, PreparedForkIndexSelection};
pub mod data;
mod projected_field_values;

#[cfg(test)]
pub(crate) use access::index_query_scratch_hint_exists;
#[cfg(test)]
pub(crate) use access::purge_index_query_scratch_hints;

#[cfg(test)]
pub(crate) use authority::last_allocation_bytes;
