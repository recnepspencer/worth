pub(crate) mod retained_definition;
pub(crate) use crate::durability::derived_index_artifacts::checkpoint_index_projections;
pub(crate) use authority::verify_checkpoint_index_bases;
pub(crate) mod access;
pub(crate) mod authority;
#[cfg(test)]
pub(crate) use authority::at_next_charges;

pub mod data;
pub(crate) mod projected_field_values;
#[cfg(test)]
pub(crate) use projected_field_values::verify_native_field_navigation;

#[cfg(test)]
pub(crate) use authority::last_allocation_bytes;
