//! Exact-slot removal requires no comparisons, Probe loans or new allocations.
use super::*;
impl<P, R, S: Summary> PredicateTree<P, R, S> {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index::predicate) fn remove(
        &mut self,
        id: WatchSlot,
    ) -> Option<Retired<P, R, S>> {
        let node = self.records.get(&id)?;
        let [left, right] = node.children;
        let next = match (left, right) {
            (Some(left), Some(right)) => {
                let mut successor = right;
                while let Some(child) = self.node(successor).children[0] {
                    successor = child;
                }
                let old_parent = self.node(successor).parent;
                let next = if old_parent == Some(id) {
                    Some(successor)
                } else {
                    let child = self.node(successor).children[1];
                    self.replace_link(successor, child);
                    self.node_mut(successor).children[1] = Some(right);
                    self.node_mut(right).parent = Some(successor);
                    old_parent
                };
                // Transplant the actual successor slot. Never move another
                // probe/receiver into the removed slot or rewrite their identity.
                self.replace_link(id, Some(successor));
                self.node_mut(successor).children[0] = Some(left);
                self.node_mut(left).parent = Some(successor);
                next
            }
            (left, right) => {
                let parent = node.parent;
                self.replace_link(id, left.or(right));
                parent
            }
        };
        let record = self
            .records
            .remove(&id)
            .expect("unlinked actual record remains owned");
        self.rebalance(next);
        Some(Retired {
            empty_backing_custody: self
                .records
                .is_empty()
                .then(|| std::mem::take(&mut self.records)),
            record_custody: Some(record),
        })
    }
}
