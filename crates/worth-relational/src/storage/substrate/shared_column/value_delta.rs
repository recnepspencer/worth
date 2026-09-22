//! Borrow changed values while pruning exact retained storage owners.
use super::{SharedColumn, PAGE_LEN};

impl<T: Clone> SharedColumn<T> {
    pub(crate) fn visit_changed_values(
        &self,
        previous: &Self,
        visit: &mut impl FnMut(usize, Option<&T>, Option<&T>),
    ) {
        walk(self, previous, self.height.max(previous.height), 0, visit);
    }
}

fn walk<T: Clone>(
    current: &SharedColumn<T>,
    previous: &SharedColumn<T>,
    height: usize,
    start: usize,
    visit: &mut impl FnMut(usize, Option<&T>, Option<&T>),
) {
    if start >= current.len.max(previous.len) {
        return;
    }
    let current_node = current.node_at(start, height);
    let previous_node = previous.node_at(start, height);
    if current_node.map(|node| node.id()) == previous_node.map(|node| node.id())
        && current.default.as_ref().map(|value| value.id())
            == previous.default.as_ref().map(|value| value.id())
        && current.len.min(start.saturating_add(PAGE_LEN << height))
            == previous.len.min(start.saturating_add(PAGE_LEN << height))
    {
        return;
    }
    if height == 0 {
        for index in start..(start + PAGE_LEN).min(current.len.max(previous.len)) {
            let new = current.get_shared(index);
            let old = previous.get_shared(index);
            if new.map(|value| value.id()) != old.map(|value| value.id()) {
                visit(
                    index,
                    new.map(|value| value.as_ref()),
                    old.map(|value| value.as_ref()),
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
