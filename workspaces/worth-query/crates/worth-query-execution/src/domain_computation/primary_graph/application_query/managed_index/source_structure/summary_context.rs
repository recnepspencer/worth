//! One actual immutable tree/image/control pair owns the entire summary invocation.
use super::super::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceRows};
use super::*;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

pub(in super::super) struct SummaryContext<
    'call,
    'source,
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    tree: &'call SourceTree,
    source: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
    control: &'call Control<'source>,
}
impl<'call, 'source, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    SummaryContext<'call, 'source, S, I>
{
    pub(in super::super) fn new(
        tree: &'call SourceTree,
        source: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
        control: &'call Control<'source>,
    ) -> Self {
        Self {
            tree,
            source,
            control,
        }
    }
    pub(super) fn get(
        &self,
        slot: Slot,
        ancestry: &mut ancestry::Ancestry,
    ) -> Result<WorthQueryManagedIndexSourceRow<'_, S, I>, Stop> {
        self.control.checkpoint(1)?;
        let node = self
            .tree
            .source_node(slot, self.control)?
            .ok_or_else(|| self.control.refuse(Stop::ForeignSummarySource))?;
        let inside = ancestry.contains(
            node,
            self.tree.root(),
            self.tree.len(),
            self.control,
            |node| self.tree.parent(node, self.control),
        )?;
        if inside {
            self.source.get(slot)
        } else {
            Err(self.control.refuse(Stop::ForeignSummarySource))
        }
    }
}
