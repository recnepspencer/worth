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
    head: Option<super::head_cell::CheckedHeadObligation>,
    previous_root: Arc<RelationalBranchRoot>,
    root_key: usize,
    generation: u64,
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
        let head = head_retention.reserve(&owner, identity, root)?;
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
                    root: Arc::clone(root),
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
            owner,
            retention_binding: self.clone(),
            head: Some(head),
            previous_root: Arc::clone(root),
            root_key,
            generation,
            terminal: false,
        })
    }
}

/// Only a completed transfer can consume retirement as a head replacement.
#[derive(Debug)]
pub(crate) struct TransferredHeadRetirement {
    retirement: RelationalHeadRetirementReservation,
}
impl TransferredHeadRetirement {
    pub(crate) fn replace_head(mut self) {
        self.retirement.retire();
    }

    pub(crate) fn replace_head_for_publication(
        mut self,
        candidate: super::RelationalCandidateRetentionObligation,
        root: Arc<RelationalBranchRoot>,
    ) -> super::RelationalPerformedSettlementObligation {
        let performed = candidate.into_performed_settlement(root, &self.retirement.owner);
        self.retirement.retire();
        performed
    }
}
impl RelationalHeadRetirementReservation {
    pub(crate) fn transfer_head(
        mut self,
        next_root: &Arc<RelationalBranchRoot>,
    ) -> TransferredHeadRetirement {
        self.head.take().expect(/* Proof: reserve_head_retirement constructs Some(CheckedHeadObligation); this non-Clone reservation is consumed by transfer_head or retire_head, so no earlier take can occur. */ "reserved head custody is consumed once").transfer(next_root);
        TransferredHeadRetirement { retirement: self }
    }
    pub(crate) fn retire_head(mut self) -> u64 {
        self.head.take().expect(/* Proof: reserve_head_retirement constructs Some(CheckedHeadObligation); this non-Clone reservation is consumed by transfer_head or retire_head, so no earlier take can occur. */ "reserved head custody is consumed once").consume();
        let root_id = self.previous_root.id();
        self.retire();
        root_id
    }
    fn retire(&mut self) {
        let mut entry = self
            .owner
            .retired_roots
            .get_mut(&self.root_key)
            .expect(/* Proof: A live retirement reservation registers this exact root key and pins its inventory entry; reclamation cannot remove an entry with live reservations. */ "reserved root remains in the concurrent inventory");
        entry.reservations = entry
            .reservations
            .checked_sub(1)
            .expect(/* Proof: Each non-Clone reservation increments once at construction and decrements once on consuming transfer or Drop; Drop is disarmed after transfer. */ "retirement reservation accounting remains balanced");
        let first_retirement = !entry.retired;
        entry.retired = true;
        drop(entry);
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
                .expect(/* Proof: A live retirement reservation registers this exact root key and pins its inventory entry; reclamation cannot remove an entry with live reservations. */ "live reservation remains in the concurrent inventory");
            entry.reservations = entry
                .reservations
                .checked_sub(1)
                .expect(/* Proof: Each non-Clone reservation increments once at construction and decrements once on consuming transfer or Drop; Drop is disarmed after transfer. */ "retirement reservation accounting remains balanced");
            entry.reservations == 0 && !entry.retired
        };
        if removable
            && let Some((_, entry)) =
                self.owner
                    .retired_roots
                    .remove_if(&self.root_key, |_, entry| {
                        entry.generation == self.generation
                            && entry.reservations == 0
                            && !entry.retired
                    })
        {
            if let Some(commit_id) = entry.root.commit_id() {
                self.owner
                    .retired_roots_by_commit
                    .remove_if(&commit_id, |_, indexed| {
                        *indexed == (self.root_key, self.generation)
                    });
            }
            release_retirement_slot(&self.owner);
        }
    }
}

impl Drop for RelationalHeadRetirementReservation {
    fn drop(&mut self) {
        self.release();
    }
}
