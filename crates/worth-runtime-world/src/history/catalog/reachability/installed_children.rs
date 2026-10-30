//! Effective installed edges, separate from forming reservation dependencies.
use super::{lock_slot, HistoryReachabilityIndex};
use crate::identity::CompositeCommitIdentity;

impl HistoryReachabilityIndex {
    fn link_installed_child(
        &mut self,
        parent: &CompositeCommitIdentity,
        child: &CompositeCommitIdentity,
    ) {
        let first = lock_slot(self.records.get(parent).expect("installed parent slot"))
            .as_ref()
            .expect("installed parent record")
            .first_child
            .clone();
        {
            let mut slot = lock_slot(self.records.get(child).expect("installed child slot"));
            let record = slot.as_mut().expect("installed child record");
            assert!(
                record.installed_parent.is_none()
                    && record.previous_sibling.is_none()
                    && record.next_sibling.is_none()
            );
            record.installed_parent = Some(parent.clone());
            record.next_sibling = first.clone();
        }
        if let Some(first) = first {
            lock_slot(self.records.get(&first).expect("first sibling slot"))
                .as_mut()
                .expect("first sibling record")
                .previous_sibling = Some(child.clone());
        }
        lock_slot(self.records.get(parent).expect("installed parent slot"))
            .as_mut()
            .expect("installed parent record")
            .first_child = Some(child.clone());
        self.mark_changed(parent);
    }

    pub(super) fn unlink_installed_child(&mut self, child: &CompositeCommitIdentity) {
        let (parent, previous, next) = {
            let mut slot = lock_slot(self.records.get(child).expect("installed child slot"));
            let record = slot.as_mut().expect("installed child record");
            (
                record.installed_parent.take(),
                record.previous_sibling.take(),
                record.next_sibling.take(),
            )
        };
        let Some(parent) = parent else {
            return;
        };
        if let Some(previous) = previous.as_ref() {
            lock_slot(self.records.get(previous).expect("previous sibling slot"))
                .as_mut()
                .expect("previous sibling record")
                .next_sibling = next.clone();
        } else {
            lock_slot(self.records.get(&parent).expect("installed parent slot"))
                .as_mut()
                .expect("installed parent record")
                .first_child = next.clone();
        }
        if let Some(next) = next.as_ref() {
            lock_slot(self.records.get(next).expect("next sibling slot"))
                .as_mut()
                .expect("next sibling record")
                .previous_sibling = previous;
        }
        self.mark_changed(&parent);
    }

    pub(super) fn install_child(
        &mut self,
        parent: &CompositeCommitIdentity,
        child: &CompositeCommitIdentity,
    ) {
        self.link_installed_child(parent, child);
    }

    pub(in crate::history::catalog) fn transfer_sole_child(
        &mut self,
        retired: &CompositeCommitIdentity,
        child: &CompositeCommitIdentity,
    ) {
        let record = self.lookup(retired).expect("retirable installed record");
        assert_eq!(record.sole_installed_child(), Some(child));
        let parent = record.installed_parent.clone();
        self.unlink_installed_child(child);
        self.unlink_installed_child(retired);
        if let Some(parent) = parent {
            self.link_installed_child(&parent, child);
        }
        self.decrement_descendant_dependency(retired);
    }
}
