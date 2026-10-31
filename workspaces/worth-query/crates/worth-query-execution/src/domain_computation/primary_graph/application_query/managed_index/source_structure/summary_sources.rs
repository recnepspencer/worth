//! Pure summary callbacks can resolve only genuine rows inside their checked region.
use super::super::WorthQueryManagedIndexSourceRow;
use super::*;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};

pub struct WorthQueryManagedIndexSummarySources<'call, 'source, S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    context: &'call SummaryContext<'call, 'source, S, I>,
    region: usize,
}
impl<'call, 'source, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryManagedIndexSummarySources<'call, 'source, S, I>
{
    pub(in super::super) fn new(
        context: &'call SummaryContext<'call, 'source, S, I>,
        region: usize,
    ) -> Self {
        Self { context, region }
    }

    pub fn get(&self, slot: Slot) -> Result<WorthQueryManagedIndexSourceRow<'_, S, I>, Stop> {
        self.context.get(self.region, slot)
    }
}
