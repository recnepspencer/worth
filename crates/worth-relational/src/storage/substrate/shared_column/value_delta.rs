//! Borrow changed values while pruning exact retained storage owners.
use super::{SharedColumn, PAGE_LEN};

impl<T: super::StoredValue> SharedColumn<T> {
    pub(crate) fn visit_changed_values(
        &self,
        previous: &Self,
        visit: &mut impl FnMut(usize, Option<&T::Value>, Option<&T::Value>),
    ) {
        walk(self, previous, self.height.max(previous.height), 0, visit);
    }
}

fn walk<T: super::StoredValue>(
    current: &SharedColumn<T>,
    previous: &SharedColumn<T>,
    height: usize,
    start: usize,
    visit: &mut impl FnMut(usize, Option<&T::Value>, Option<&T::Value>),
) {
    if start >= current.len.max(previous.len) {
        return;
    }
    let current_node = current.node_at(start, height);
    let previous_node = previous.node_at(start, height);
    if current_node.map(|node| node.id()) == previous_node.map(|node| node.id())
        && match (&current.default, &previous.default) {
            (None, None) => true,
            (Some(a), Some(b)) => a.same_storage(b),
            _ => false,
        }
        && current.len.min(start.saturating_add(PAGE_LEN << height))
            == previous.len.min(start.saturating_add(PAGE_LEN << height))
    {
        return;
    }
    if height == 0 {
        for index in start..(start + PAGE_LEN).min(current.len.max(previous.len)) {
            let new = current.get_shared(index);
            let old = previous.get_shared(index);
            if match (new, old) {
                (None, None) => false,
                (Some(a), Some(b)) => !a.same_storage(b),
                _ => true,
            } {
                visit(
                    index,
                    new.map(super::StoredValue::value),
                    old.map(super::StoredValue::value),
                );
            }
        }
    } else {
        walk(current, previous, height - 1, start, visit);
        walk(
            current,
            previous,
            height - 1,
            start + (PAGE_LEN << (height - 1)),
            visit,
        );
    }
}
