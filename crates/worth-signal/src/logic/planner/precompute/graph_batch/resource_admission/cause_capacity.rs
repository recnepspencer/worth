//! Selected consumer scope copies admitted before an epoch evaluator runs.
use std::collections::BTreeMap;

use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::ObservedCauseScope;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

pub(super) struct CauseCandidate {
    pub(super) growth: u64,
    pub(super) next_scopes: Vec<AcceptedCauseScope>,
    pub(super) new_scope_entries: usize,
}

pub(super) struct AcceptedCauseScope {
    pub(super) consumer: NodeId,
    pub(super) count: usize,
    pub(super) heap: u64,
    pub(super) scoped_count: usize,
}

pub(super) fn measure(
    graph: &SignalGraph,
    producer: NodeId,
    consumers: &[NodeId],
    prior_causes: &BTreeMap<NodeId, (usize, u64, usize)>,
    work: &mut Work,
    budget: &mut SignalPreparationBudget,
) -> Result<CauseCandidate, SignalError> {
    budget.claim_vec::<AcceptedCauseScope>(consumers.len())?;
    let observed_mark = budget.checkpoint();
    budget.claim_vec::<ObservedCauseScope>(consumers.len())?;
    let selected = graph.epoch_selected_cause_growth_bound(producer, consumers, work, budget)?;
    let observed = graph.epoch_selected_cause_scope_payload_bound(producer, consumers, work)?;
    let mut new_scopes = Vec::with_capacity(observed.len());
    let mut growth = selected;
    let mut new_scope_entries = 0usize;
    for ObservedCauseScope {
        consumer,
        old_count,
        incoming_count,
        incoming_heap,
        old_scopes,
        incoming_scopes,
    } in observed
    {
        let (prior_count, prior_heap, prior_scopes) =
            prior_causes.get(&consumer).copied().unwrap_or((0, 0, 0));
        if !prior_causes.contains_key(&consumer) {
            new_scope_entries += 1;
        }
        let pending = old_count.checked_add(prior_count).ok_or_else(overflow)?;
        let total = pending.checked_add(incoming_count).ok_or_else(overflow)?;
        // The cause owner preallocates pending plus all relevant edges before
        // copying. Replacements and removals can only reduce this capacity.
        let mut inline = slots::<ResolvedDependencyCause>(total)?;
        // A zero- or one-cause set is already normalized and keeps its
        // admitted input Vec. Larger sets may need both sort scratch arrays.
        if total > 1 {
            inline = inline
                .checked_add(slots::<(usize, ResolvedDependencyCause)>(total)?)
                .and_then(|bytes| bytes.checked_add(slots::<ResolvedDependencyCause>(total).ok()?))
                .ok_or_else(overflow)?;
        }
        let cache_scopes = old_scopes
            .checked_add(prior_scopes)
            .and_then(|n| n.checked_add(incoming_scopes))
            .ok_or_else(overflow)?;
        inline = inline
            .checked_add(slots::<(
                crate::data::aspect::Aspect,
                crate::data::output::PartitionSubscription,
            )>(cache_scopes)?)
            .ok_or_else(overflow)?;
        // Each selected new/prior cause may own its canonical changed-scope
        // Vec through constructor, pending copy, fold and final store. The
        // cache backing above is a separate allocation with its own lifetime.
        let cause_scope_slots = prior_scopes
            .checked_add(incoming_scopes)
            .ok_or_else(overflow)?;
        inline = inline
            .checked_add(
                slots::<crate::data::output::PartitionSubscription>(cause_scope_slots)?
                    .checked_mul(4)
                    .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
        growth = growth
            .checked_add(inline)
            .and_then(|bytes| {
                bytes.checked_add(incoming_heap.checked_add(prior_heap)?.checked_mul(4)?)
            })
            .ok_or_else(overflow)?;
        new_scopes.push(AcceptedCauseScope {
            consumer,
            count: prior_count
                .checked_add(incoming_count)
                .ok_or_else(overflow)?,
            heap: prior_heap.checked_add(incoming_heap).ok_or_else(overflow)?,
            scoped_count: prior_scopes
                .checked_add(incoming_scopes)
                .ok_or_else(overflow)?,
        });
    }
    budget.release(observed_mark);
    Ok(CauseCandidate {
        growth,
        next_scopes: new_scopes,
        new_scope_entries,
    })
}

fn slots<T>(count: usize) -> Result<u64, SignalError> {
    u64::try_from(count)
        .map_err(|_| overflow())?
        .checked_mul(std::mem::size_of::<T>() as u64)
        .ok_or_else(overflow)
}

fn overflow() -> SignalError {
    SignalError::invalid_input("selected cause scope bound overflow")
}
