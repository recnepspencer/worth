use super::node::{ColumnNode, ColumnStorage};
use super::{SharedColumn, PAGE_LEN};
use crate::storage::substrate::StorageAllocation as Arc;

impl<T: Clone> SharedColumn<T> {
    /// Restore a dense owned column without replaying each element through the
    /// persistent mutation path. The resulting pages retain ordinary COW rules.
    pub(super) fn from_dense_values(values: Vec<T>) -> Self {
        let len = values.len();
        if len == 0 {
            return Self::default();
        }

        let mut values = values.into_iter();
        let mut level = Vec::with_capacity(len.div_ceil(PAGE_LEN));
        while values.len() != 0 {
            let mut page = Vec::with_capacity(PAGE_LEN);
            for value in values.by_ref().take(PAGE_LEN) {
                page.push(Some(Arc::new(value)));
            }
            let value_count = page.len();
            page.resize_with(PAGE_LEN, || None);
            level.push(Arc::new(ColumnNode {
                storage: ColumnStorage::Page(page),
                node_count: 1,
                page_count: 1,
                value_count,
            }));
        }

        let mut height = 0;
        while level.len() > 1 {
            let mut parents = Vec::with_capacity(level.len().div_ceil(2));
            let mut children = level.into_iter();
            while let Some(left) = children.next() {
                parents.push(Arc::new(ColumnNode::branch(Some(left), children.next())));
            }
            level = parents;
            height += 1;
        }
        Self {
            root: level.pop(),
            height,
            len,
            default: None,
        }
    }
}
