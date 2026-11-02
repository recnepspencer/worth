mod acquisition;
pub(crate) use acquisition::RelationalRetentionRequestAcquisitionDenial;
mod record;
mod root_counts;
mod roots;
use record::RelationalRetentionObligationRecord;
use root_counts::release_root;
use roots::{RetainedRoots, RootIds};
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapCell};
mod retained_release;
mod retirement;
#[cfg(test)]
pub(crate) use retained_release::assert_external_cut_and_lane_busy;
pub(crate) use retained_release::{
    RelationalRetentionRetirementCustody, RelationalRetentionRetirementError,
};

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
    obligations: ExecutionOrderedMap<u64, RelationalRetentionObligationRecord>,
    root_obligations: ExecutionOrderedMap<u64, usize>,
    active_operations: u64,
    counters: RelationalRetentionCostCounters,
    interruption_counters: crate::runtime::RelationalInterruptionCostCounters,
}

#[derive(Debug)]
pub(crate) struct RelationalRetentionGuard {
    pub(super) owner: Weak<RelationalBranchRetentionOwnerInner>,
    lane: Arc<RelationalBranchRetentionLane>,
    owner_identity: usize,
    obligation_id: u64,
    roots: RetainedRoots,
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
            .root_obligations
            .contains_key(&root_id)
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
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
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

    pub(super) fn transfer_to_performed_settlement(
        &mut self,
        current_root: Arc<RelationalBranchRoot>,
        owner: &Arc<RelationalBranchRetentionOwnerInner>,
    ) {
        let mut state = lock_lane(&self.lane);
        let RelationalBranchRetentionLaneState {
            obligations,
            root_obligations,
            ..
        } = &mut *state;
        let record = obligations
            .get(&self.obligation_id)
            .expect(/* Proof: The private live candidate token owns this registered obligation until consuming transfer or Drop; no other token can remove its ID. */ "live candidate obligation remains registered through transfer");
        // Proof: only RelationalCandidateRetentionObligation::into_performed_settlement calls this method. Its private acquire hardcodes Candidate; into_performed_settlement consumes that non-Clone token, so it cannot transfer twice.
        assert_eq!(
            /* Proof: RelationalRetentionGuard is non-Clone and owns one private obligation id. acquire inserts the exact root ids; consuming transfer updates the record and held roots together. release removes that id once under the lane Mutex and terminal prevents a second release. */
            record.kind(),
            RelationalRetentionObligationKind::Candidate
        );
        let original_ids = record.root_ids();
        let mut removed_counts: [Option<ExecutionOrderedMapCell<u64, usize>>; 2] = [None, None];
        let retained_position = original_ids.iter().position(|id| *id == current_root.id()).expect(/* Proof: Candidate acquisition holds the prepared root which consuming publication installs as the current root before settlement transfer. */ "performed root is held by the candidate");
        for (position, root) in original_ids.iter().enumerate() {
            if position != retained_position {
                removed_counts[position] = release_root(root_obligations, *root);
            }
        }
        record.transfer_settlement(current_root.id());
        record_release(
            &mut state.counters,
            RelationalRetentionObligationKind::Candidate,
        );
        owner
            .ordinary_counters
            .record_release(RelationalRetentionObligationKind::Candidate);
        record_acquire(
            &mut state.counters,
            RelationalRetentionObligationKind::PerformedSettlement,
        );
        owner
            .ordinary_counters
            .record_acquire(RelationalRetentionObligationKind::PerformedSettlement);
        drop(state);
        drop(removed_counts);
        self.roots.clear();
        self.roots.push(current_root);
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
        let retired = state.release_obligation(self.obligation_id, &self.roots, &owner);
        drop(state);
        drop(retired);
        drop(std::mem::take(&mut self.roots));
        super::RelationalBranchRetentionTerminalOutcome::Released
    }
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

impl RelationalBranchRetentionBinding {
    pub(in crate::history::retention) fn same_lane(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.owner, &other.owner) && Arc::ptr_eq(&self.lane, &other.lane)
    }
}
