//! Result backing keeps its original execution grant when transferred out of the group.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryGroupCompletionKind {
    Recorded,
    AlreadyRecorded,
}

pub struct WorthQueryHistoryGroupCompletions<S: ApplicationSchema, I: ApplicationMutationIntent<S>>
{
    items: Vec<(WorthQueryClosedOperation, Completion<S, I>)>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>>
    WorthQueryHistoryGroupCompletions<S, I>
{
    pub(super) fn new(
        items: Vec<(WorthQueryClosedOperation, Completion<S, I>)>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
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
    pub fn iter(&self) -> std::slice::Iter<'_, (WorthQueryClosedOperation, Completion<S, I>)> {
        self.items.iter()
    }
    pub fn iter_mut(
        &mut self,
    ) -> std::slice::IterMut<'_, (WorthQueryClosedOperation, Completion<S, I>)> {
        self.items.iter_mut()
    }
}

pub struct WorthQueryHistoryGroupCompletionIter<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
> {
    items: std::vec::IntoIter<(WorthQueryClosedOperation, Completion<S, I>)>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> IntoIterator
    for WorthQueryHistoryGroupCompletions<S, I>
{
    type Item = (WorthQueryClosedOperation, Completion<S, I>);
    type IntoIter = WorthQueryHistoryGroupCompletionIter<S, I>;
    fn into_iter(self) -> Self::IntoIter {
        WorthQueryHistoryGroupCompletionIter {
            items: self.items.into_iter(),
            _memory: self._memory,
        }
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> Iterator
    for WorthQueryHistoryGroupCompletionIter<S, I>
{
    type Item = (WorthQueryClosedOperation, Completion<S, I>);
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> ExactSizeIterator
    for WorthQueryHistoryGroupCompletionIter<S, I>
{
}
