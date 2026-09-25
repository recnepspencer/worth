use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use crate::branch::{RelationalBranchIdentity, RelationalBranchRoot};

use super::cost_counters::{record_acquire, record_release};
use super::owner::{
    release_live_obligation_capacity, reserve_live_obligation_capacity,
    RelationalBranchRetentionOwnerInner,
};
use super::{
    RelationalRetentionAcquisitionDenial, RelationalRetentionCostCounters,
    RelationalRetentionObligationKind,
};

#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRetentionBinding {
    pub(super) owner: Weak<RelationalBranchRetentionOwnerInner>,
    lane: Arc<RelationalBranchRetentionLane>,
}

#[derive(Debug)]
struct RelationalBranchRetentionLane {
    identity: Option<RelationalBranchIdentity>,
    state: Mutex<RelationalBranchRetentionLaneState>,
}

#[derive(Debug, Default)]
struct RelationalBranchRetentionLaneState {
    next_obligation_id: u64,
    obligations: HashMap<u64, RelationalRetentionObligationRecord>,
    active_operations: u64,
    counters: RelationalRetentionCostCounters,
    interruption_counters: crate::runtime::RelationalInterruptionCostCounters,
}

#[derive(Debug)]
struct RelationalRetentionObligationRecord {
    kind: RelationalRetentionObligationKind,
    root_ids: Vec<u64>,
    active_operation: bool,
}

#[derive(Debug)]
pub(crate) struct RelationalRetentionGuard {
    owner: Weak<RelationalBranchRetentionOwnerInner>,
    lane: Arc<RelationalBranchRetentionLane>,
    owner_identity: usize,
    obligation_id: u64,
    roots: Vec<Arc<RelationalBranchRoot>>,
    terminal: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalRetentionOwnerRelationship {
    SameOwner,
    OwnerUnavailable,
    DifferentOwner,
}

impl RelationalBranchRetentionBinding {
    pub(crate) fn has_root_obligation(&self, root_id: u64) -> bool {
        lock_lane(&self.lane)
            .obligations
            .values()
            .any(|obligation| obligation.root_ids.contains(&root_id))
    }

    pub(super) fn new(
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
        identity: Option<RelationalBranchIdentity>,
    ) -> Self {
        Self {
            owner: Arc::downgrade(owner),
            lane: Arc::new(RelationalBranchRetentionLane {
                identity,
                state: Mutex::new(RelationalBranchRetentionLaneState::default()),
            }),
        }
    }

    pub(crate) fn acquire(
        &self,
        kind: RelationalRetentionObligationKind,
        roots: Vec<Arc<RelationalBranchRoot>>,
        operation_branch: Option<RelationalBranchIdentity>,
    ) -> Result<RelationalRetentionGuard, RelationalRetentionAcquisitionDenial> {
        if roots.is_empty() || roots.len() > 2 {
            return Err(RelationalRetentionAcquisitionDenial::RootSetTooLarge);
        }
        let owner = self
            .owner
            .upgrade()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        if let Some(operation_branch) = operation_branch.as_ref() {
            if self.lane.identity.as_ref() != Some(operation_branch) {
                return Err(RelationalRetentionAcquisitionDenial::OwnerUnavailable);
            }
        }
        reserve_live_obligation_capacity(&owner)?;
        let mut state = lock_lane(&self.lane);
        let obligation_id = match state.next_obligation_id.checked_add(1) {
            Some(id) => id,
            None => {
                drop(state);
                release_live_obligation_capacity(&owner);
                return Err(RelationalRetentionAcquisitionDenial::IdentityExhausted);
            }
        };
        state.next_obligation_id = obligation_id;
        let active_operation = operation_branch.is_some();
        if active_operation {
            state.active_operations = match state.active_operations.checked_add(1) {
                Some(count) => count,
                None => {
                    drop(state);
                    release_live_obligation_capacity(&owner);
                    return Err(RelationalRetentionAcquisitionDenial::IdentityExhausted);
                }
            };
        }
        state.obligations.insert(
            obligation_id,
            RelationalRetentionObligationRecord {
                kind,
                root_ids: roots.iter().map(|root| root.id()).collect(),
                active_operation,
            },
        );
        record_acquire(&mut state.counters, kind);
        owner.ordinary_counters.record_acquire(kind);
        Ok(RelationalRetentionGuard {
            owner: Arc::downgrade(&owner),
            lane: Arc::clone(&self.lane),
            owner_identity: Arc::as_ptr(&owner) as usize,
            obligation_id,
            roots,
            terminal: false,
        })
    }

    pub(crate) fn install_head(
        &self,
        identity: RelationalBranchIdentity,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<super::RelationalHeadRetentionObligation, RelationalRetentionAcquisitionDenial>
    {
        let owner = self
            .owner
            .upgrade()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        if identity.runtime_instance_id() != owner.runtime_instance_id {
            return Err(RelationalRetentionAcquisitionDenial::OwnerUnavailable);
        }
        super::owner::reserve_live_head_capacity(&owner)?;
        owner
            .live_head_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        owner
            .head_install_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(super::RelationalHeadRetentionObligation::new(
            &owner,
            identity,
            Arc::clone(root),
        ))
    }

    pub(crate) fn active_operation_count(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<u64, RelationalRetentionAcquisitionDenial> {
        self.owner
            .upgrade()
            .ok_or(RelationalRetentionAcquisitionDenial::OwnerUnavailable)?;
        if self.lane.identity.as_ref() != Some(identity) {
            return Err(RelationalRetentionAcquisitionDenial::OwnerUnavailable);
        }
        Ok(lock_lane(&self.lane).active_operations)
    }

    pub(crate) fn counters(&self) -> RelationalRetentionCostCounters {
        lock_lane(&self.lane).counters
    }

    pub(crate) fn interruption_counters(
        &self,
    ) -> crate::runtime::RelationalInterruptionCostCounters {
        lock_lane(&self.lane).interruption_counters
    }

    pub(crate) fn record_interruption(&self, event: crate::runtime::RelationalInterruptionEvent) {
        lock_lane(&self.lane).interruption_counters.record(event);
    }

    pub(crate) fn record_head_install(&self) {
        let mut state = lock_lane(&self.lane);
        state.counters.head_installs = state.counters.head_installs.saturating_add(1);
    }

    pub(crate) fn record_head_transfer(&self) {
        let mut state = lock_lane(&self.lane);
        state.counters.head_transfers = state.counters.head_transfers.saturating_add(1);
    }

    pub(crate) fn record_retired_root_enqueue(&self) {
        let mut state = lock_lane(&self.lane);
        state.counters.retired_root_enqueues =
            state.counters.retired_root_enqueues.saturating_add(1);
    }
}

impl RelationalRetentionGuard {
    pub(crate) fn record_interruption(&self, event: crate::runtime::RelationalInterruptionEvent) {
        lock_lane(&self.lane).interruption_counters.record(event);
    }
    pub(crate) fn owner_relationship(
        &self,
        binding: &RelationalBranchRetentionBinding,
    ) -> RelationalRetentionOwnerRelationship {
        let Some(owner) = self.owner.upgrade() else {
            return RelationalRetentionOwnerRelationship::OwnerUnavailable;
        };
        let Some(binding_owner) = binding.owner.upgrade() else {
            return RelationalRetentionOwnerRelationship::OwnerUnavailable;
        };
        if Arc::ptr_eq(&owner, &binding_owner)
            && Arc::as_ptr(&owner) as usize == self.owner_identity
        {
            RelationalRetentionOwnerRelationship::SameOwner
        } else {
            RelationalRetentionOwnerRelationship::DifferentOwner
        }
    }

    pub(crate) fn release_explicitly(&mut self) -> super::RelationalBranchRetentionTerminalOutcome {
        self.release()
    }

    pub(crate) fn transfer_to_performed_settlement(
        &mut self,
        current_root: Arc<RelationalBranchRoot>,
    ) {
        let mut state = lock_lane(&self.lane);
        let record = state
            .obligations
            .get_mut(&self.obligation_id)
            .expect("live candidate obligation remains registered through transfer");
        assert_eq!(record.kind, RelationalRetentionObligationKind::Candidate);
        record.kind = RelationalRetentionObligationKind::PerformedSettlement;
        record.root_ids = vec![current_root.id()];
        record_release(
            &mut state.counters,
            RelationalRetentionObligationKind::Candidate,
        );
        owner_counter(&self.owner)
            .ordinary_counters
            .record_release(RelationalRetentionObligationKind::Candidate);
        record_acquire(
            &mut state.counters,
            RelationalRetentionObligationKind::PerformedSettlement,
        );
        owner_counter(&self.owner)
            .ordinary_counters
            .record_acquire(RelationalRetentionObligationKind::PerformedSettlement);
        drop(state);
        drop(std::mem::replace(&mut self.roots, vec![current_root]));
    }

    fn release(&mut self) -> super::RelationalBranchRetentionTerminalOutcome {
        if self.terminal {
            return super::RelationalBranchRetentionTerminalOutcome::Released;
        }
        self.terminal = true;
        let Some(owner) = self.owner.upgrade() else {
            self.roots.clear();
            return super::RelationalBranchRetentionTerminalOutcome::OwnerUnavailable;
        };
        let mut state = lock_lane(&self.lane);
        if let Some(record) = state.obligations.remove(&self.obligation_id) {
            debug_assert_eq!(
                record.root_ids,
                self.roots.iter().map(|root| root.id()).collect::<Vec<_>>()
            );
            if record.active_operation {
                state.active_operations = state
                    .active_operations
                    .checked_sub(1)
                    .expect("active branch-operation accounting remains balanced");
            }
            record_release(&mut state.counters, record.kind);
            owner.ordinary_counters.record_release(record.kind);
            release_live_obligation_capacity(&owner);
        }
        drop(state);
        drop(std::mem::take(&mut self.roots));
        super::RelationalBranchRetentionTerminalOutcome::Released
    }
}

fn owner_counter(
    owner: &Weak<RelationalBranchRetentionOwnerInner>,
) -> Arc<RelationalBranchRetentionOwnerInner> {
    owner
        .upgrade()
        .expect("live retention guard keeps its owner available")
}

impl Drop for RelationalRetentionGuard {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

fn lock_lane(
    lane: &RelationalBranchRetentionLane,
) -> std::sync::MutexGuard<'_, RelationalBranchRetentionLaneState> {
    lane.state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
