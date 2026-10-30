//! Pure summary callbacks can resolve only genuine rows inside their checked region.
use super::super::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceRows};
use super::*;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

pub struct WorthQueryManagedIndexSummarySources<'call, 'source, S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    tree: &'call SourceTree,
    region: usize,
    source: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
    control: &'call Control<'source>,
}
impl<'call, 'source, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryManagedIndexSummarySources<'call, 'source, S, I>
{
    pub(in super::super) fn new(
        tree: &'call SourceTree,
        region: usize,
        source: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
        control: &'call Control<'source>,
    ) -> Self {
        Self {
            tree,
            region,
            source,
            control,
        }
    }

    pub fn get(&self, slot: Slot) -> Result<WorthQueryManagedIndexSourceRow<'_, S, I>, Stop> {
        self.control.checkpoint(1)?;
        maps::lookup(self.control, self.tree.sources.len(), size_of::<Slot>())?;
        let mut current = self.tree.sources.get(&slot).copied();
        let mut visited = 0;
        while let Some(node) = current {
            if node == self.region {
                return self.source.get(slot);
            }
            if visited >= self.tree.len() {
                break;
            }
            visited += 1;
            current = self.tree.parent(node, self.control)?;
        }
        Err(self.control.refuse(Stop::ForeignSummarySource))
    }
}
