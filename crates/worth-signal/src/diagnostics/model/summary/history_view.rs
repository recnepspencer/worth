use super::ExecutionHistorySummary;
use crate::diagnostics::state::DiagnosticHistory;

/// Borrowed retained execution history. Owned export is an explicit traversal.
#[derive(Debug, Clone, Copy)]
pub struct RetainedExecutionHistoryView<'a> {
    history: &'a DiagnosticHistory<ExecutionHistorySummary>,
}

impl<'a> RetainedExecutionHistoryView<'a> {
    pub(crate) fn new(history: &'a DiagnosticHistory<ExecutionHistorySummary>) -> Self {
        Self { history }
    }
    pub fn len(&self) -> usize {
        self.history.len()
    }
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = &'a ExecutionHistorySummary> + ExactSizeIterator {
        self.history.iter()
    }
    pub fn first(&self) -> Option<&'a ExecutionHistorySummary> {
        self.history.front()
    }
    pub fn last(&self) -> Option<&'a ExecutionHistorySummary> {
        self.history.back()
    }
}
