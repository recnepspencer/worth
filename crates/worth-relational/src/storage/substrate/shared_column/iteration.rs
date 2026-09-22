use super::node::{ColumnNode, ColumnStorage};
use super::PAGE_LEN;
use crate::storage::substrate::StorageAllocation as Arc;

pub(crate) struct SharedColumnIter<'a, T: Clone> {
    pending: Vec<(Option<&'a ColumnNode<T>>, usize)>,
    page: std::slice::Iter<'a, Option<Arc<T>>>,
    default: Option<&'a T>,
    default_run: usize,
    remaining: usize,
    root: Option<&'a ColumnNode<T>>,
    height: usize,
    back: usize,
}

impl<'a, T: Clone> SharedColumnIter<'a, T> {
    pub(super) fn new(
        root: Option<&'a ColumnNode<T>>,
        height: usize,
        remaining: usize,
        default: Option<&'a T>,
    ) -> Self {
        Self {
            pending: vec![(root, height)],
            page: [].iter(),
            default,
            default_run: 0,
            remaining,
            root,
            height,
            back: remaining,
        }
    }
}

impl<'a, T: Clone> Iterator for SharedColumnIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        loop {
            if self.default_run != 0 {
                self.default_run -= 1;
                self.remaining -= 1;
                return Some(
                    self.default
                        .expect("unmaterialized column range needs a default"),
                );
            }
            if let Some(value) = self.page.next() {
                self.remaining -= 1;
                return Some(
                    value
                        .as_deref()
                        .or(self.default)
                        .expect("column value must exist"),
                );
            }
            let (node, height) = self.pending.pop()?;
            match node.map(|node| &node.storage) {
                None => self.default_run = (PAGE_LEN << height).min(self.remaining),
                Some(ColumnStorage::Page(values)) => self.page = values.iter(),
                Some(ColumnStorage::Branch(children)) => self.pending.extend(
                    children
                        .iter()
                        .rev()
                        .map(|child| (child.as_deref(), height - 1)),
                ),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: Clone> ExactSizeIterator for SharedColumnIter<'_, T> {}

impl<'a, T: Clone> DoubleEndedIterator for SharedColumnIter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.back -= 1;
        self.root
            .and_then(|root| root.get(self.back, self.height))
            .map(Arc::as_ref)
            .or(self.default)
    }
}
