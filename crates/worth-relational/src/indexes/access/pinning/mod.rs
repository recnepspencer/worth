//! Installed entries borrow the live owner only to select immutable custody.
//! Sibling read algorithms cannot access the owner field.
use super::{DerivedIndexDefinitionLookup, PreparedEntityFieldLookup};
use crate::runtime::RelationalRuntime;
#[path = "../admitted_generation.rs"]
mod admitted_generation;
mod bounded_field_input;
mod definition_inventory;
pub(super) use bounded_field_input::sealed;
pub use bounded_field_input::BoundedEntityFieldLookupRead;
mod field_preparation;
mod generation_observation;
mod generation_reads;
mod related_ordering;
mod relation_join;
mod snapshot_field;
pub struct IndexAccess<'runtime> {
    runtime: &'runtime RelationalRuntime,
}
impl<'runtime> IndexAccess<'runtime> {
    pub(super) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }

    #[cfg(test)]
    pub(crate) fn runtime(&self) -> &'runtime RelationalRuntime {
        self.runtime
    }
}
