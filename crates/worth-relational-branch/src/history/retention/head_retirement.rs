use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::branch::identity::RelationalBranchIdentity;
use crate::branch::root::RelationalBranchRoot;
use std::collections::btree_map::Entry;

use super::owner::{
    release_retirement_slot, reserve_retirement_slot, RelationalBranchRetentionOwnerInner,
    RelationalRetiredBranchRoot,
};
use super::{
    RelationalBranchHeadRetentionCell, RelationalBranchRetentionBinding,
    RelationalRetentionAcquisitionDenial,
};

#[derive(Debug)]
pub struct RelationalHeadRetirementReservation {
    owner: Arc<RelationalBranchRetentionOwnerInner>,
    retention_binding: RelationalBranchRetentionBinding,
    head: Option<super::head_cell::CheckedHeadObligation>,
    previous_root: Arc<RelationalBranchRoot>,
    root_key: usize,
    generation: u64,
    terminal: bool,
}

/// Capacity can be reserved before the current head is compared. It grants no transfer.
#[derive(Debug)]
pub struct RelationalHeadRetirementCapacity {
    reservation: RelationalHeadRetirementReservation,
}
impl RelationalHeadRetirementCapacity {
    /// Call under branch coordination, after comparing the selected head.
    pub fn bind_current_head(
        self,
    ) -> Result<RelationalHeadRetirementReservation, RelationalRetentionAcquisitionDenial> {
        self.reservation
            .head
            .as_ref()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?
            .validate_root(&self.reservation.previous_root)?;
        Ok(self.reservation)
    }
}

impl RelationalBranchRetentionBinding {
    pub fn reserve_head_retirement(
        &self,
        identity: &RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
        head_retention: &Arc<RelationalBranchHeadRetentionCell>,
    ) -> Result<RelationalHeadRetirementReservation, RelationalRetentionAcquisitionDenial> {
        self.reserve_head_retirement_capacity(identity, root, head_retention)?
            .bind_current_head()
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub fn reserve_head_retirement_capacity(
        &self,
        identity: &RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
        head_retention: &Arc<RelationalBranchHeadRetentionCell>,
    ) -> Result<RelationalHeadRetirementCapacity, RelationalRetentionAcquisitionDenial> {
        let owner = self
            .owner
            .upgrade()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        let head = head_retention.reserve(&owner, identity)?;
        let branch_binding = head_retention.binding()?;
        let root_key = Arc::as_ptr(root) as usize;
        let mut replaced_binding = None;
        let mut roots = owner.write_retired_roots();
        let mut inserted = false;
        let generation = match roots.entry(root_key) {
            Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .reservations
                    .acquire()
                    .map_err(|_| RelationalRetentionAcquisitionDenial::IdentityExhausted)?;
                replaced_binding = entry
                    .get_mut()
                    .branch_bindings
                    .insert(identity.branch_id().clone(), branch_binding);
                entry.get().generation
            }
            Entry::Vacant(entry) => {
                let generation = reserve_retirement_slot(&owner)?;
                let mut reservations = worth_foundational::DagReferenceCount::default();
                reservations.acquire().expect("first reservation fits");
                entry.insert(RelationalRetiredBranchRoot {
                    root: Arc::clone(root),
                    branch_bindings: std::collections::BTreeMap::from([(
                        identity.branch_id().clone(),
                        branch_binding,
                    )]),
                    historical_binding: RelationalBranchRetentionBinding::new(&owner, None),
                    reservations,
                    dirty: Default::default(),
                    retired: false,
                    generation,
                });
                inserted = true;
                generation
            }
        };
        drop(roots);
        drop(replaced_binding);
        if inserted {
            owner
                .retired_root_locators
                .register(root, root_key, generation);
        }
        Ok(RelationalHeadRetirementCapacity {
            reservation: RelationalHeadRetirementReservation {
                owner,
                retention_binding: self.clone(),
                head: Some(head),
                previous_root: Arc::clone(root),
                root_key,
                generation,
                terminal: false,
            },
        })
    }
}

/// Only a completed transfer can consume retirement as a head replacement.
#[derive(Debug)]
pub struct TransferredHeadRetirement {
    retirement: RelationalHeadRetirementReservation,
}
impl TransferredHeadRetirement {
    pub fn replace_head(mut self) {
        self.retirement.retire();
    }

    pub fn replace_head_for_publication(
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
    pub fn transfer_head(
        mut self,
        next_root: &Arc<RelationalBranchRoot>,
    ) -> TransferredHeadRetirement {
        self.head.take().expect(/* Proof: reserve_head_retirement_capacity installs Some(CheckedHeadObligation), and bind_current_head moves that non-Clone reservation without taking it. Only consuming transfer_head or retire_head takes the obligation. */ "reserved head custody is consumed once").transfer(next_root);
        TransferredHeadRetirement { retirement: self }
    }
    pub fn retire_head(mut self) -> u64 {
        self.head.take().expect(/* Proof: reserve_head_retirement_capacity installs Some(CheckedHeadObligation), and bind_current_head moves that non-Clone reservation without taking it. Only consuming transfer_head or retire_head takes the obligation. */ "reserved head custody is consumed once").consume();
        let root_id = self.previous_root.id();
        self.retire();
        root_id
    }
    fn retire(&mut self) {
        let mut roots = self.owner.write_retired_roots();
        let entry = roots.get_mut(&self.root_key)
            .expect(/* Proof: A live retirement reservation registers this exact root key and pins its inventory entry; reclamation cannot remove an entry with live reservations. */ "reserved root remains in the concurrent inventory");
        entry.reservations.release();
        let first_retirement = !entry.retired;
        entry.retired = true;
        if first_retirement {
            self.owner
                .retired_root_count
                .fetch_add(1, Ordering::Release);
            roots.mark_changed(self.root_key);
        }
        drop(roots);
        if first_retirement {
            self.retention_binding.record_retired_root_enqueue();
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
            let mut roots = self.owner.write_retired_roots();
            let entry = roots.get_mut(&self.root_key)
                .expect(/* Proof: A live retirement reservation registers this exact root key and pins its inventory entry; reclamation cannot remove an entry with live reservations. */ "live reservation remains in the concurrent inventory");
            entry.reservations.release();
            entry.reservations.count() == 0 && !entry.retired
        };
        let removed = if removable {
            let mut roots = self.owner.write_retired_roots();
            if roots.get(&self.root_key).is_some_and(|entry| {
                entry.generation == self.generation
                    && entry.reservations.count() == 0
                    && !entry.retired
            }) {
                roots.remove(&self.root_key)
            } else {
                None
            }
        } else {
            None
        };
        if let Some(entry) = removed {
            self.owner
                .retired_root_locators
                .remove(&entry.root, self.root_key, self.generation);
            release_retirement_slot(&self.owner);
        }
    }
}

impl Drop for RelationalHeadRetirementReservation {
    fn drop(&mut self) {
        self.release();
    }
}
