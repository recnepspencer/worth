//! Checked AVL node carrier and region-confined genuine Source row lookup.

use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexNodeId, WorthQueryManagedIndexSourceRow,
    WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop,
    WorthQueryManagedIndexSummarySources,
};

/// Opaque AVL node retained in Query's checked binary source tree.
#[derive(Copy, Debug)]
pub struct WorthQueryOrderedIndexNode<Summary> {
    pub(crate) source: WorthQueryManagedIndexSourceSlot,
    pub(crate) own_summary: Summary,
    pub(crate) summary: Summary,
    pub(crate) left: Option<WorthQueryManagedIndexNodeId>,
    pub(crate) right: Option<WorthQueryManagedIndexNodeId>,
    pub(crate) height: u8,
}

impl<Summary: Copy> Clone for WorthQueryOrderedIndexNode<Summary> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Borrowed genuine Source row lookup for summary combination and pruning.
pub struct WorthQueryOrderedIndexSources<'call, 'source, S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    backing: SourcesBacking<'call, 'source, S, I>,
}

enum SourcesBacking<'call, 'source, S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    Rows(&'call WorthQueryManagedIndexSourceRows<'source, S, I>),
    Summary(&'call WorthQueryManagedIndexSummarySources<'call, 'source, S, I>),
}

impl<'call, 'source, S, I> WorthQueryOrderedIndexSources<'call, 'source, S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    pub(crate) fn from_rows(rows: &'call WorthQueryManagedIndexSourceRows<'source, S, I>) -> Self {
        Self {
            backing: SourcesBacking::Rows(rows),
        }
    }

    pub(crate) fn from_summary(
        sources: &'call WorthQueryManagedIndexSummarySources<'call, 'source, S, I>,
    ) -> Self {
        Self {
            backing: SourcesBacking::Summary(sources),
        }
    }

    /// Resolves a genuine source slot confined to the active checked region.
    pub fn get(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
    ) -> Result<WorthQueryManagedIndexSourceRow<'_, S, I>, WorthQueryManagedIndexStop> {
        match &self.backing {
            SourcesBacking::Rows(rows) => rows.get(slot),
            SourcesBacking::Summary(sources) => sources.get(slot),
        }
    }
}
