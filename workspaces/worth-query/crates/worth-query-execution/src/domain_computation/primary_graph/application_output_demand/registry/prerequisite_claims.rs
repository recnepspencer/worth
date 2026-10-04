mod cancellation;
mod evidence_handoff;
mod prerequisite_capacity;
mod prerequisite_denials;
use prerequisite_capacity::validate_predecessors;
use prerequisite_denials::{capacity_denial, closed_denial, coverage_denial, work_denial};
use std::sync::Arc;

use super::{
    prerequisite_work::predecessor_lookup_work, required_context::RequiredOutputDemandContext,
    DemandState, WorthQueryOutputDemandKey, WorthQueryOutputDemandRegistry,
};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, RecordedSettlementIdentity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};

/// Claims the exact previously settled inputs of one running output before its
/// product effect. The successful publication moves these claims into the
/// demand record; dropping the ticket reverses them.
pub(in crate::domain_computation::primary_graph) struct PreparedPrerequisiteClaims {
    context: RequiredOutputDemandContext,
    predecessors: Vec<Arc<WorthQueryOutputDemandKey>>,
    predecessor_slots_bytes: usize,
    reserved_identity: Option<Arc<RecordedSettlementIdentity>>,
    reserved_posting: Option<super::settlement_index::Posting>,
    reserved_cleanup: Option<Box<super::settlement_index::PendingVacancyCleanup>>,
    published: bool,
}

impl RequiredOutputDemandContext {
    pub(in crate::domain_computation::primary_graph) fn prepare_prerequisites<'a>(
        self,
        inputs: impl ExactSizeIterator<Item = &'a Arc<RecordedSettlementIdentity>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedPrerequisiteClaims, WorthQueryOutputDemandDenial> {
        if self.work_membership().is_some() {
            // The performed path can require a local Full cue after World has
            // committed. Its token and queue link were admitted with the row.
            admission
                .charge_external_work(8)
                .map_err(|_| work_denial())?;
        }
        let count = inputs.len();
        let slots_bytes = count
            .checked_mul(std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(slots_bytes as u64)
            .map_err(|_| capacity_denial())?;
        admission
            .charge_external_work(count as u64)
            .map_err(|_| work_denial())?;
        let mut predecessors = Vec::new();
        predecessors
            .try_reserve_exact(count)
            .map_err(|_| capacity_denial())?;
        admission
            .charge_external_work(
                self.key()
                    .producer
                    .len()
                    .checked_add(1)
                    .ok_or_else(work_denial)? as u64,
            )
            .map_err(|_| work_denial())?;
        let owner = WorthQueryOutputDemandRegistry::clone(self.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let downstream = self.key();
        let record = state.records.get(downstream).ok_or_else(closed_denial)?;
        if !matches!(record.state, DemandState::Running) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                "required output is no longer the running producer operation",
            ));
        }
        if record.prepared_prerequisite_claims != 0 {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::SchedulingDeferred,
                "output already has a prepared prerequisite publication",
            ));
        }
        for identity in inputs {
            let upstream = state
                .settlement_keys
                .get_exact_admitted(identity, admission)?
                .ok_or_else(prerequisite_denials::stale_upstream_denial)?;
            if upstream.as_ref() == downstream {
                return Err(coverage_denial());
            }
            predecessors.push(upstream);
        }
        let maximum_key_work = predecessors
            .iter()
            .map(|key| key.producer.len())
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(work_denial)?;
        let comparisons = count
            .checked_mul(usize::BITS as usize - count.max(1).leading_zeros() as usize + 1)
            .and_then(|visits| visits.checked_mul(maximum_key_work))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(comparisons as u64)
            .map_err(|_| work_denial())?;
        predecessors.sort_unstable();
        predecessors.dedup();
        let lookup_work = predecessor_lookup_work(&state, &predecessors, maximum_key_work)?;
        admission
            .charge_ordered_operations(1, lookup_work as u64)
            .map_err(|_| work_denial())?;
        let predecessor_slots_bytes = predecessors
            .capacity()
            .checked_mul(std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>())
            .ok_or_else(capacity_denial)?;
        let additional_members = validate_predecessors(&state, &predecessors)?;
        admission
            .charge_external_work(
                u64::try_from(predecessors.len().checked_mul(4).ok_or_else(work_denial)?)
                    .map_err(|_| work_denial())?,
            )
            .map_err(|_| work_denial())?;
        let settlements = &state.records.get(downstream).unwrap().settlements;
        let old_settlement_capacity = settlements.capacity();
        let projected_settlement_capacity = if settlements.len() < old_settlement_capacity {
            old_settlement_capacity
        } else {
            old_settlement_capacity
                .checked_mul(2)
                .map(|capacity| capacity.max(4))
                .ok_or_else(capacity_denial)?
        };
        let settlement_slot_bytes = std::mem::size_of::<(Arc<RecordedSettlementIdentity>, usize)>();
        let settlement_growth = projected_settlement_capacity
            .checked_sub(old_settlement_capacity)
            .and_then(|slots| slots.checked_mul(settlement_slot_bytes))
            .ok_or_else(capacity_denial)?;
        if settlement_growth != 0 {
            let copy_work = settlements.len();
            admission
                .charge_external_work(copy_work as u64)
                .map_err(|_| work_denial())?;
        }
        let prior_keys = &state.records.get(downstream).unwrap().prerequisites;
        admission
            .charge_external_work(prior_keys.len() as u64)
            .map_err(|_| work_denial())?;
        let prior_key_work = prior_keys.iter().try_fold(0usize, |maximum, key| {
            1_usize
                .checked_add(key.producer.len())
                .map(|bytes| maximum.max(bytes))
                .ok_or_else(work_denial)
        })?;
        let minimum_member_bytes = super::required_members::minimum_member_bytes()
            .filter(|bytes| *bytes != 0)
            .ok_or_else(work_denial)?;
        // Other interests may enter after preparation. The named registry cap
        // bounds every later ordered-tree height until this ticket publishes.
        let required_member_ceiling = state.required_budget_bytes / minimum_member_bytes;
        let release_work = super::prerequisite_work::prior_release_work(
            required_member_ceiling,
            prior_keys.len(),
            prior_key_work,
        )?;
        admission
            .charge_ordered_operations(1, release_work as u64)
            .map_err(|_| work_denial())?;
        // Dropping this ticket can remove the newly selected memberships
        // after other interests have grown the tree. Reserve that exact
        // selected rollback under the same finite registry-height ceiling.
        let rollback_work = super::prerequisite_work::prior_release_work(
            required_member_ceiling,
            predecessors.len(),
            maximum_key_work,
        )?;
        admission
            .charge_ordered_operations(1, rollback_work as u64)
            .map_err(|_| work_denial())?;
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        let claimed = predecessor_slots_bytes
            .checked_add(additional_members)
            .and_then(|bytes| bytes.checked_add(settlement_growth))
            .ok_or_else(capacity_denial)?;
        let required = state
            .required_reserved_bytes
            .checked_add(claimed)
            .ok_or_else(capacity_denial)?;
        if !state.has_required_capacity(required) {
            return Err(super::required_custody::full_custody_denial(
                claimed,
                state.required_budget_bytes,
            ));
        }
        // Vec growth holds the old backing until the replacement is ready.
        let settlement_preparation = if settlement_growth == 0 {
            0
        } else {
            projected_settlement_capacity
                .checked_mul(settlement_slot_bytes)
                .ok_or_else(capacity_denial)?
        };
        let owner_growth = additional_members
            .checked_add(settlement_preparation)
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(owner_growth as u64)
            .map_err(|_| capacity_denial())?;
        let settlements = &mut state.records.get_mut(downstream).unwrap().settlements;
        settlements
            .try_reserve_exact(1)
            .map_err(|_| capacity_denial())?;
        let actual_settlement_growth = settlements
            .capacity()
            .checked_sub(old_settlement_capacity)
            .and_then(|slots| slots.checked_mul(settlement_slot_bytes))
            .ok_or_else(capacity_denial)?;
        assert!(
            actual_settlement_growth <= settlement_growth,
            "admitted settlement slots"
        );
        state.required_reserved_bytes += actual_settlement_growth;
        for upstream in &predecessors {
            let prepared = state
                .prepare_required_member(upstream)
                .expect("all prerequisite members were admitted together");
            state
                .records
                .get_mut(upstream.as_ref())
                .unwrap()
                .framework_required_count += 1;
            state.install_required_member(prepared);
        }
        state
            .records
            .get_mut(downstream)
            .unwrap()
            .prepared_prerequisite_claims += 1;
        state.required_reserved_bytes += predecessor_slots_bytes;
        drop(state);
        Ok(PreparedPrerequisiteClaims {
            context: self,
            predecessors,
            predecessor_slots_bytes,
            reserved_identity: None,
            reserved_posting: None,
            reserved_cleanup: None,
            published: false,
        })
    }
}

impl PreparedPrerequisiteClaims {
    pub(in crate::domain_computation::primary_graph) fn work_membership(
        &self,
    ) -> Option<Arc<super::required_work::RequiredWorkMembership>> {
        self.context.work_membership()
    }
    pub(in crate::domain_computation::primary_graph) fn retains_reserved_identity(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> bool {
        self.reserved_identity
            .as_ref()
            .is_some_and(|held| Arc::ptr_eq(held, identity))
            && self.reserved_posting.is_some()
            && self.reserved_cleanup.is_some()
    }
}

impl PreparedPrerequisiteClaims {
    /// The lineage owner has already reserved and minted this exact address.
    /// Insert its invisible registry vacancy before any World owner effect.
    pub(in crate::domain_computation::primary_graph) fn reserve_identity(
        &mut self,
        identity: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        assert!(self.reserved_identity.is_none());
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (posting, cleanup) = state.reserve_settlement_vacancy(identity, admission)?;
        self.reserved_identity = Some(Arc::clone(identity));
        self.reserved_posting = Some(posting);
        self.reserved_cleanup = Some(cleanup);
        Ok(())
    }

    /// Prepare the successor's invisible posting while the prior posting is
    /// still held. A denial leaves every old token with this ticket; after a
    /// successful reserve, exchanging and queueing the old cue cannot fail.
    pub(in crate::domain_computation::primary_graph) fn replace_reserved_identity_for_recovery(
        &mut self,
        identity: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (posting, cleanup) = state.reserve_settlement_vacancy(identity, admission)?;
        let previous = self.reserved_cleanup.replace(cleanup);
        self.reserved_posting = Some(posting);
        self.reserved_identity = Some(Arc::clone(identity));
        if let Some(previous) = previous {
            state.defer_cancelled_settlement_vacancy(previous);
        }
        Ok(())
    }

    /// This is called only after the product publication has committed. All
    /// allocations and index capacity were admitted by `prepare_prerequisites`.
    pub(in crate::domain_computation::primary_graph) fn publish(
        mut self,
        identity: Arc<RecordedSettlementIdentity>,
    ) -> super::SupersededSettlements {
        let owner = WorthQueryOutputDemandRegistry::clone(self.context.registry());
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let downstream = self.context.key();
        let expected = self
            .reserved_identity
            .take()
            .expect("managed output reserved its exact settlement before effects");
        super::settlement_index::SettlementIndex::fill_prepared(
            self.reserved_posting
                .as_ref()
                .expect("prepared settlement posting remains retained"),
            expected.as_ref(),
            Arc::clone(&identity),
            Arc::clone(self.context.key_arc()),
        );
        let record = state
            .records
            .get_mut(downstream)
            .expect("prepared demand stays pinned");
        assert_eq!(record.prepared_prerequisite_claims, 1);
        record.prepared_prerequisite_claims = 0;
        assert!(record.settlements.len() < record.settlements.capacity());
        record
            .settlements
            .push((identity, self.context.retained_bytes()));
        let old = std::mem::replace(
            &mut record.prerequisites,
            std::mem::take(&mut self.predecessors),
        );
        let released_slots = old.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
        for upstream in &old {
            let prior = state
                .records
                .get_mut(upstream.as_ref())
                .expect("settled prerequisite retained");
            prior.framework_required_count -= 1;
            state.remove_required_member_if_released(upstream);
        }
        for upstream in &old {
            state.defer_terminal_cleanup(upstream, 0);
        }
        state.defer_terminal_cleanup(self.context.key_arc(), 0);
        state.required_reserved_bytes =
            state.required_reserved_bytes.saturating_sub(released_slots);
        // The context's owned key is now held by the exact-settlement index.
        self.context.transfer_custody_to_registry();
        self.predecessor_slots_bytes = 0;
        self.reserved_cleanup.take();
        self.published = true;
        drop(state);
        drop(old);
        let evicted = self.context.take_evicted_settlement();
        super::SupersededSettlements::new(owner, Arc::clone(self.context.key_arc()), evicted)
    }
}
