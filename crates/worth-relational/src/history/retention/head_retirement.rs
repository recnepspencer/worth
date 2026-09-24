use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};
use dashmap::mapref::entry::Entry;

use super::owner::{
    release_retirement_slot, reserve_retirement_slot, RelationalBranchRetentionOwnerInner,
    RelationalRetiredBranchRoot,
};
use super::{
    RelationalBranchHeadRetentionCell, RelationalBranchRetentionBinding,
    RelationalRetentionAcquisitionDenial,
};

#[derive(Debug)]
pub(crate) struct RelationalHeadRetirementReservation {
    owner: Arc<RelationalBranchRetentionOwnerInner>,
    retention_binding: RelationalBranchRetentionBinding,
    head_retention: Arc<RelationalBranchHeadRetentionCell>,
    owner_identity: usize,
    identity: RelationalBranchIdentity,
    root_key: usize,
    generation: u64,
    head_transferred: bool,
    terminal: bool,
}

impl RelationalBranchRetentionBinding {
    pub(crate) fn reserve_head_retirement(
        &self,
        identity: &RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
        head_retention: &Arc<RelationalBranchHeadRetentionCell>,
    ) -> Result<RelationalHeadRetirementReservation, RelationalRetentionAcquisitionDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        let branch_binding = head_retention.binding()?;
        let root_key = Arc::as_ptr(root) as usize;
        let generation = match owner.retired_roots.entry(root_key) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .branch_bindings
                    .insert(identity.branch_id().clone(), branch_binding);
                entry.get_mut().reservations = entry
                    .get()
                    .reservations
                    .checked_add(1)
                    .ok_or(RelationalRetentionAcquisitionDenial::IdentityExhausted)?;
                entry.get().generation
            }
            Entry::Vacant(entry) => {
                let generation = reserve_retirement_slot(&owner)?;
                entry.insert(RelationalRetiredBranchRoot {
                    root: Some(Arc::clone(root)),
                    branch_bindings: std::collections::BTreeMap::from([(
                        identity.branch_id().clone(),
                        branch_binding,
                    )]),
                    historical_binding: RelationalBranchRetentionBinding::new(&owner, None),
                    reservations: 1,
                    retired: false,
                    generation,
                });
                if let Some(commit_id) = root.commit_id() {
                    owner
                        .retired_roots_by_commit
                        .insert(commit_id, (root_key, generation));
                }
                generation
            }
        };
        Ok(RelationalHeadRetirementReservation {
            owner_identity: Arc::as_ptr(&owner) as usize,
            owner,
            retention_binding: self.clone(),
            head_retention: Arc::clone(head_retention),
            identity: identity.clone(),
            root_key,
            generation,
            head_transferred: false,
            terminal: false,
        })
    }
}

impl RelationalHeadRetirementReservation {
    pub(crate) fn transfer_head(
        &mut self,
        previous_root: &Arc<RelationalBranchRoot>,
        next_root: &Arc<RelationalBranchRoot>,
    ) {
        self.require_reserved_root(previous_root);
        assert!(!self.terminal && !self.head_transferred);
        self.head_retention.transfer(
            self.owner_identity,
            &self.identity,
            previous_root,
            next_root,
        );
        self.head_transferred = true;
    }

    pub(crate) fn replace_head(mut self, previous_root: Arc<RelationalBranchRoot>) {
        assert!(
            self.head_transferred,
            "head obligation transfers at cutover"
        );
        self.retire(previous_root);
    }

    pub(crate) fn retire_head(mut self, previous_root: Arc<RelationalBranchRoot>) -> u64 {
        self.require_reserved_root(&previous_root);
        self.head_retention
            .consume(self.owner_identity, &self.identity, &previous_root);
        let root_id = previous_root.id();
        self.retire(previous_root);
        root_id
    }

    fn retire(&mut self, root: Arc<RelationalBranchRoot>) {
        self.require_reserved_root(&root);
        let mut entry = self
            .owner
            .retired_roots
            .get_mut(&self.root_key)
            .expect("reserved root remains in the concurrent inventory");
        entry.reservations = entry
            .reservations
            .checked_sub(1)
            .expect("retirement reservation accounting remains balanced");
        let first_retirement = !entry.retired;
        entry.retired = true;
        drop(entry);
        drop(root);
        if first_retirement {
            self.owner
                .retired_root_count
                .fetch_add(1, Ordering::Release);
            self.retention_binding.record_retired_root_enqueue();
            self.owner
                .retired_root_order
                .push((self.root_key, self.generation));
            self.owner
                .retired_enqueue_count
                .fetch_add(1, Ordering::Relaxed);
        }
        self.terminal = true;
    }

    fn require_reserved_root(&self, root: &Arc<RelationalBranchRoot>) {
        assert_eq!(Arc::as_ptr(root) as usize, self.root_key);
    }

    fn release(&mut self) {
        if self.terminal {
            return;
        }
        self.terminal = true;
        let removable = {
            let mut entry = self
                .owner
                .retired_roots
                .get_mut(&self.root_key)
                .expect("live reservation remains in the concurrent inventory");
            entry.reservations = entry
                .reservations
                .checked_sub(1)
                .expect("retirement reservation accounting remains balanced");
            entry.reservations == 0 && !entry.retired
        };
        if removable {
            if let Some((_, entry)) =
                self.owner
                    .retired_roots
                    .remove_if(&self.root_key, |_, entry| {
                        entry.generation == self.generation
                            && entry.reservations == 0
                            && !entry.retired
                    })
            {
                if let Some(root) = entry.root {
                    if let Some(commit_id) = root.commit_id() {
                        self.owner
                            .retired_roots_by_commit
                            .remove_if(&commit_id, |_, indexed| {
                                *indexed == (self.root_key, self.generation)
                            });
                    }
                }
                release_retirement_slot(&self.owner);
            }
        }
    }
}

impl Drop for RelationalHeadRetirementReservation {
    fn drop(&mut self) {
        self.release();
    }
}
