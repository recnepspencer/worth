use crate::data::retained_storage::RetainedStorageBacking;
use std::sync::Arc;

use super::fork_page::ForkPage;

pub(crate) enum PersistentVectorIter<'a, T: Clone, const PAGE_LEN: usize> {
    Exclusive(std::slice::Iter<'a, T>),
    ForkShared {
        base: &'a Arc<RetainedStorageBacking<Vec<T>>>,
        changed_pages: &'a im::OrdMap<usize, Arc<ForkPage<T>>>,
        len: usize,
        next: usize,
    },
}

impl<'a, T: Clone, const PAGE_LEN: usize> Iterator for PersistentVectorIter<'a, T, PAGE_LEN> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Exclusive(values) => values.next(),
            Self::ForkShared {
                base,
                changed_pages,
                len,
                next,
            } => {
                if *next >= *len {
                    return None;
                }
                let index = *next;
                *next += 1;
                let page_index = index / PAGE_LEN;
                changed_pages.get(&page_index).map_or_else(
                    || base.get(index),
                    |page| page.get(base, index, index % PAGE_LEN),
                )
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = match self {
            Self::Exclusive(values) => values.len(),
            Self::ForkShared { len, next, .. } => *len - *next,
        };
        (remaining, Some(remaining))
    }
}

impl<T: Clone, const PAGE_LEN: usize> ExactSizeIterator for PersistentVectorIter<'_, T, PAGE_LEN> {}
