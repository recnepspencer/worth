//! One actual immutable tree/image/control pair owns the entire summary invocation.
use super::super::{WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceRows};
use super::*;
use std::cell::RefCell;
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
    execution: ExecutionRequest<'call, 'call>,
    ancestry: RefCell<ancestry::Ancestry>,
}
impl<'call, 'source, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    SummaryContext<'call, 'source, S, I>
{
    pub(in super::super) fn new(
        tree: &'call SourceTree,
        source: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
        control: &'call Control<'source>,
        execution: ExecutionRequest<'call, 'call>,
    ) -> Self {
        Self {
            tree,
            source,
            control,
            execution,
            ancestry: RefCell::default(),
        }
    }
    pub(super) fn get(
        &self,
        region: usize,
        slot: Slot,
    ) -> Result<WorthQueryManagedIndexSourceRow<'_, S, I>, Stop> {
        self.control.checkpoint(1)?;
        let node = self
            .tree
            .source_node(slot, self.control)?
            .ok_or_else(|| self.control.refuse(Stop::ForeignSummarySource))?;
        let inside = self.ancestry.borrow_mut().contains(
            region,
            node,
            self.tree.root(),
            self.tree.len(),
            &self.execution,
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
