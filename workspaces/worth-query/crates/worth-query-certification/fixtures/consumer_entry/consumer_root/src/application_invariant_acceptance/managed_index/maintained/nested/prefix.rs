//! This schema binds its actual Source field to the shared prefix provider.
use super::{query, schema::Catalog};
#[cfg(test)]
use crate::application_invariant_acceptance::managed_index::prefix_observation::Calls;
use crate::application_invariant_acceptance::managed_index::prefix_provider::{
    self, PrefixSourceIndex,
};
pub use prefix_provider::Provider;
#[cfg(test)]
use prefix_provider::Stop;
#[cfg(test)]
use std::sync::atomic::Ordering;
#[cfg(test)]
use worth_query_consumer_values::PrefixIndexDenial;
use worth_query_host::facade::primary_graph::*;
mod definition;
pub use definition::PrefixIndex;
#[cfg(test)]
pub(super) mod source_updates;
#[cfg(test)]
pub(super) mod text_read;

impl PrefixSourceIndex<Catalog> for PrefixIndex {
    fn with_name<R>(
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
        visitor: impl FnOnce(&str) -> R,
    ) -> Result<R, WorthQueryManagedIndexTextReadDenial> {
        row.with_text(query::name(), visitor)
    }
    #[cfg(test)]
    fn inspect_text(
        calls: &Calls,
        row: &WorthQueryManagedIndexSourceRow<'_, Catalog, Self>,
    ) -> Result<(), Stop> {
        text_read::inspect(calls, row)
    }
    #[cfg(test)]
    fn observe_update(
        calls: &Calls,
        before: &WorthQueryManagedIndexSourceRows<'_, Catalog, Self>,
        after: &WorthQueryManagedIndexSourceRows<'_, Catalog, Self>,
        changes: &[WorthQueryManagedIndexSourceChange],
    ) -> Result<(), Stop> {
        source_updates::record(calls, before, after, changes)
    }
}
