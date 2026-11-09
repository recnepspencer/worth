//! Immutable authoring history shared by transaction reads and later stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalTransactionStagedMutations<I: Clone>(im::Vector<I>);

impl<I: Clone> Default for RelationalTransactionStagedMutations<I> {
    fn default() -> Self {
        Self(im::Vector::new())
    }
}
impl<I: Clone> RelationalTransactionStagedMutations<I> {
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &I> {
        self.0.iter()
    }
    pub(super) fn push(&mut self, intent: I) {
        self.0.push_back(intent);
    }
    pub(super) fn suffix(&self, applied: usize) -> impl Iterator<Item = &I> {
        (applied..self.0.len()).map(|i| &self.0[i])
    }
}
