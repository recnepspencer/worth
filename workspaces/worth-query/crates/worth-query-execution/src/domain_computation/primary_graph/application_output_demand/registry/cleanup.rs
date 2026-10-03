use std::sync::Arc;

use super::{
    DemandRecord, DemandRegistryState, WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

impl DemandRegistryState {
    /// The selected Arc already owns this key. Linking it into the registry is
    /// allocation free, so a post-effect pin release can only defer cleanup.
    pub(super) fn defer_terminal_cleanup(
        &mut self,
        key: &Arc<WorthQueryOutputDemandKey>,
        retained_key_bytes: usize,
    ) -> bool {
        let Some(record) = self.records.get_mut(key.as_ref()) else {
            return false;
        };
        if !record.terminal()
            || record.framework_required_count != 0
            || record.prepared_prerequisite_claims != 0
            || record.pending_cleanup_queued
            || (record.prerequisites.capacity() == 0
                && record.settlements.capacity() == 0
                && record.performed_obligations.capacity() == 0)
        {
            return false;
        }
        record.pending_cleanup_next = self.pending_cleanup_head.take();
        record.pending_cleanup_queued = true;
        record.pending_cleanup_key_bytes = retained_key_bytes;
        self.pending_cleanup_head = Some(Arc::clone(key));
        true
    }

    /// Detach one exact queued row. The caller destroys this custody after
    /// releasing the registry guard, then may advance to the next head.
    fn take_terminal_cleanup_step(
        &mut self,
        admission: &mut InvalidationEditAdmission,
        entry_paid: bool,
    ) -> Result<Option<RetiredTerminalCleanup>, WorthQueryOutputDemandDenial> {
        if !entry_paid {
            admission
                .charge_external_work(2)
                .map_err(|_| empty_work_denial())?;
        }
        let Some(head) = self.pending_cleanup_head.as_ref() else {
            return Ok(None);
        };
        // Every reached denial subject in this owner and its record,
        // prerequisite, and settlement index helpers is at most 64 bytes.
        // Fund that one terminal String before any later refusal can build it.
        let terminal_backing = 64usize
            .checked_add(std::mem::size_of::<String>())
            .ok_or_else(empty_work_denial)?;
        admission
            .charge_external_work(u64::try_from(terminal_backing).map_err(|_| empty_work_denial())?)
            .map_err(|_| empty_work_denial())?;
        admission
            .admit_read_scratch(u64::try_from(terminal_backing).map_err(|_| empty_work_denial())?)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    empty_work_denial()
                }
                _ => empty_capacity_denial(),
            })?;
        admission
            .charge_external_work(key_work(head)?)
            .map_err(|_| work_denial())?;
        // Pay every reached record-tree descent before unlinking the head.
        // The final removal's node shifts have a separate owner quote.
        for _ in 0..5 {
            self.charge_record_lookup(head.as_ref(), admission)?;
        }
        self.charge_record_removal_movement(admission)?;
        let record = self
            .records
            .get(head.as_ref())
            .expect("queued row stays retained");
        let release_obligations = record.terminal()
            && record.framework_required_count == 0
            && record.prepared_prerequisite_claims == 0
            && record.performed_obligations.capacity() != 0;
        if release_obligations {
            let slot = std::mem::size_of::<super::PerformedOutputObligation>();
            let destruction = record
                .performed_obligations
                .capacity()
                .checked_mul(slot)
                .and_then(|backing| {
                    record
                        .performed_obligations
                        .len()
                        .checked_mul(slot)
                        .and_then(|initialized| backing.checked_add(initialized))
                })
                .and_then(|work| work.checked_add(8))
                .ok_or_else(work_denial)?;
            admission
                .charge_external_work(u64::try_from(destruction).map_err(|_| work_denial())?)
                .map_err(|_| work_denial())?;
            for _ in 0..3 {
                self.charge_record_lookup(head.as_ref(), admission)?;
            }
            super::required_work::charge_required_key_lookup(self, head.as_ref(), admission)?;
            let levels =
                usize::BITS as usize - self.required_keys.len().max(1).leading_zeros() as usize;
            let movement = levels
                .checked_mul(3)
                .and_then(|nodes| {
                    nodes.checked_mul(super::required_members::minimum_member_bytes()?)
                })
                .and_then(|work| work.checked_add(4))
                .ok_or_else(work_denial)?;
            admission
                .charge_ordered_operations(1, u64::try_from(movement).map_err(|_| work_denial())?)
                .map_err(|_| work_denial())?;
        }
        let edges = record.prerequisites.len();
        let identities = record.settlements.len();
        let selected_slots = edges.checked_add(identities).ok_or_else(work_denial)?;
        admission
            .charge_external_work(selected_slots as u64)
            .map_err(|_| work_denial())?;
        let maximum_key_work = record
            .prerequisites
            .iter()
            .map(|key| key_work(key))
            .try_fold(0u64, |maximum, bytes| bytes.map(|bytes| maximum.max(bytes)))?;
        let navigation = super::prerequisite_work::prior_release_work(
            self.records.len().max(self.required_keys.len()),
            edges,
            usize::try_from(maximum_key_work).map_err(|_| work_denial())?,
        )?;
        admission
            .charge_ordered_operations(edges as u64, navigation as u64)
            .map_err(|_| work_denial())?;
        // Each upstream edge reaches a record decrement, two membership
        // record probes, and a terminal-defer record probe. A released key
        // may then rotate/merge three required-set nodes at each level.
        let required_count = self.required_keys.len();
        let required_levels = usize::BITS as usize - required_count.max(1).leading_zeros() as usize;
        let required_node =
            super::required_members::minimum_member_bytes().ok_or_else(work_denial)?;
        for upstream in &record.prerequisites {
            for _ in 0..4 {
                self.charge_record_lookup(upstream, admission)?;
            }
            super::required_work::charge_required_key_lookup(self, upstream, admission)?;
            let movement = required_levels
                .checked_mul(3)
                .and_then(|nodes| nodes.checked_mul(required_node))
                .and_then(|work| work.checked_add(4))
                .ok_or_else(work_denial)?;
            admission
                .charge_ordered_operations(1, u64::try_from(movement).map_err(|_| work_denial())?)
                .map_err(|_| work_denial())?;
        }
        self.settlement_keys.admit_selected_removal_work(
            record.settlements.iter().map(|(identity, _)| identity),
            admission,
        )?;
        let member_backing = edges
            .checked_mul(std::mem::size_of::<
                super::required_members::DetachedRequiredMember,
            >())
            .ok_or_else(work_denial)?;
        admission
            .admit_read_scratch(u64::try_from(member_backing).map_err(|_| work_denial())?)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    work_denial()
                }
                _ => empty_capacity_denial(),
            })?;
        let mut retired_members = Vec::new();
        if edges != 0 {
            retired_members
                .try_reserve_exact(edges)
                .map_err(|_| empty_capacity_denial())?;
            if retired_members.capacity() != edges {
                return Err(empty_capacity_denial());
            }
        }
        // The queued key must be destroyed before its membership's final
        // capacity ticket is released, even if this removes the row.
        let detached_work = std::mem::size_of::<RetiredTerminalCleanup>()
            .checked_add(std::mem::size_of::<DemandRecord>())
            .and_then(|work| {
                work.checked_add(std::mem::size_of::<Vec<Arc<WorthQueryOutputDemandKey>>>())
            })
            .and_then(|work| work.checked_add(member_backing))
            .and_then(|work| work.checked_add(8))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(detached_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        // This drain immediately inspects the next head, including the empty
        // case. Pay that final header read before unlinking this head so a
        // one-unit-short request leaves the current queued work retriable.
        admission
            .charge_external_work(2)
            .map_err(|_| empty_work_denial())?;
        let member = record.work_membership.as_ref().map(Arc::clone);

        let head = self.pending_cleanup_head.take().unwrap();
        let record = self.records.get_mut(head.as_ref()).unwrap();
        self.pending_cleanup_head = record.pending_cleanup_next.take();
        record.pending_cleanup_queued = false;
        let retained_key_bytes = std::mem::take(&mut record.pending_cleanup_key_bytes);
        let release = record.terminal()
            && record.framework_required_count == 0
            && record.prepared_prerequisite_claims == 0;
        let (released, settlements, release_refund) = if release {
            self.release_record_prerequisites_detached(head.as_ref(), &mut retired_members)
        } else {
            (Vec::new(), Vec::new(), 0)
        };
        let (obligations, refund_obligation_bytes, own_member) = if release_obligations {
            let record = self.records.get_mut(head.as_ref()).unwrap();
            let refund = record.obligation_reserved_bytes();
            let obligations = std::mem::take(&mut record.performed_obligations);
            let member = self.remove_required_member_if_released_detached(head.as_ref());
            (obligations, refund, member)
        } else {
            (
                Vec::new(),
                0,
                super::required_members::DetachedRequiredMember::default(),
            )
        };
        let refund_required_bytes = retained_key_bytes
            .checked_add(release_refund)
            .and_then(|bytes| bytes.checked_add(own_member.refund_required_bytes))
            .expect("admitted terminal cleanup charge fits");
        let removed = if self.records.get(head.as_ref()).is_some_and(|record| {
            record.terminal()
                && record.interests == 0
                && !record.is_required()
                && record.performed_source.is_none()
                && !record.pending_cleanup_queued
        }) {
            self.records.remove(head.as_ref())
        } else {
            None
        };
        Ok(Some(RetiredTerminalCleanup {
            _released: released,
            _settlements: settlements,
            _retired_members: retired_members,
            _obligations: obligations,
            _own_member: own_member,
            _removed: removed,
            _head: head,
            _member: member,
            refund_required_bytes,
            refund_obligation_bytes,
        }))
    }
}

/// Field order preserves the key and removed row through their physical
/// destruction before the retained membership can refund its capacity.
struct RetiredTerminalCleanup {
    _released: Vec<Arc<WorthQueryOutputDemandKey>>,
    _settlements: Vec<(
        Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>,
        usize,
    )>,
    _retired_members: Vec<super::required_members::DetachedRequiredMember>,
    _obligations: Vec<super::PerformedOutputObligation>,
    _own_member: super::required_members::DetachedRequiredMember,
    _removed: Option<DemandRecord>,
    _head: Arc<WorthQueryOutputDemandKey>,
    _member: Option<Arc<super::required_work::RequiredWorkMembership>>,
    refund_required_bytes: usize,
    refund_obligation_bytes: usize,
}

impl WorthQueryOutputDemandRegistry {
    pub(super) fn drain_terminal_cleanup_admitted(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut entry_paid = false;
        loop {
            let retired = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                state.take_terminal_cleanup_step(admission, entry_paid)?
            };
            let Some(retired) = retired else {
                return Ok(());
            };
            let refund = retired.refund_required_bytes;
            let obligation_refund = retired.refund_obligation_bytes;
            drop(retired);
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.required_reserved_bytes = state.required_reserved_bytes.saturating_sub(refund);
            state.obligation_reserved_bytes = state
                .obligation_reserved_bytes
                .saturating_sub(obligation_refund);
            entry_paid = true;
        }
    }
}

#[cfg(test)]
impl DemandRegistryState {
    pub(super) fn drain_terminal_cleanup(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut entry_paid = false;
        while let Some(retired) = self.take_terminal_cleanup_step(admission, entry_paid)? {
            let refund = retired.refund_required_bytes;
            let obligation_refund = retired.refund_obligation_bytes;
            drop(retired);
            self.required_reserved_bytes = self.required_reserved_bytes.saturating_sub(refund);
            self.obligation_reserved_bytes = self
                .obligation_reserved_bytes
                .saturating_sub(obligation_refund);
            entry_paid = true;
        }
        Ok(())
    }
}

impl DemandRecord {
    pub(super) fn terminal(&self) -> bool {
        matches!(&self.state, super::DemandState::Failed(_))
            || matches!(&self.state, super::DemandState::Output(output)
                if matches!(output.advancement, super::WorthQueryOutputAdvancement::Stopped { .. }))
    }
}

fn key_work(key: &WorthQueryOutputDemandKey) -> Result<u64, WorthQueryOutputDemandDenial> {
    1_usize
        .checked_add(key.producer.len())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(work_denial)
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "queued terminal prerequisite cleanup exceeds request work",
    )
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "",
    )
}
