//! Result backing retains its original execution grant through owned iteration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryGroupCompletionKind {
    Recorded,
    AlreadyRecorded,
}
pub struct WorthQueryHistoryGroupCompletions<T> {
    items: Vec<T>,
    _memory: Vec<worth_execution::ExecutionMemoryReservation>,
}
impl<T> WorthQueryHistoryGroupCompletions<T> {
    pub(super) fn new(
        items: Vec<T>,
        memory: Vec<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            items,
            _memory: memory,
        }
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.items.iter_mut()
    }
}
pub struct WorthQueryHistoryGroupCompletionIter<T> {
    items: std::vec::IntoIter<T>,
    _memory: Vec<worth_execution::ExecutionMemoryReservation>,
}
impl<T> IntoIterator for WorthQueryHistoryGroupCompletions<T> {
    type Item = T;
    type IntoIter = WorthQueryHistoryGroupCompletionIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        WorthQueryHistoryGroupCompletionIter {
            items: self.items.into_iter(),
            _memory: self._memory,
        }
    }
}
impl<T> Iterator for WorthQueryHistoryGroupCompletionIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}
impl<T> ExactSizeIterator for WorthQueryHistoryGroupCompletionIter<T> {}
#[cfg(test)]
#[path = "results_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
