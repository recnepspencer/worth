//! One cause-slot and waiter draft for all producer changes in a graph epoch.
use std::collections::BTreeMap;

use super::node_publication::PreparedCauseNodeReplacement;
use super::{PreparedDirectCauseNodes, SignalGraph, Work};
use crate::data::aspect::AspectMask;
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::{
    EpochCauseStoreEdit, NormalizedCauseSet, PendingCauseSetId, PreparedCauseSlot,
    PreparedEpochCauseStore,
};
use crate::data::graph::{PendingRevalidationNodeProjection, PreparedDependencyTopologyEpoch};
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::logic::evaluation::EvaluationWork;

use super::super::{
    EpochCauseHead, PreparedConsumerCauseSet, PreparedDirectCounterDeltas,
    PreparedEpochDirectCauseAdmission,
};

pub(crate) struct PreparedEpochDirectCausePublication {
    nodes: PreparedDirectCauseNodes,
    stores: PreparedEpochDirectCauseStores,
    suppressed_downstream: u64,
}

pub(crate) struct PreparedEpochDirectCauseStores {
    commits: Vec<ProducedAspectDelta>,
    edits: Vec<EpochCauseStoreEdit>,
    finals: Vec<(PendingCauseSetId, NormalizedCauseSet)>,
    counters: PreparedDirectCounterDeltas,
}

pub(crate) struct PreparedEpochCausePublication {
    store: PreparedEpochCauseStore,
    counters: PreparedDirectCounterDeltas,
}

impl SignalGraph {
    /// Fixed epoch cause projection, slot and replacement containers. Waiters
    /// enter the projection map; only direct consumers receive cause slots and
    /// replacement payloads.
    pub(crate) fn epoch_cause_shape_bound(
        producers: usize,
        direct_consumers: usize,
        waiter_projections: usize,
        transitions: usize,
    ) -> Result<u64, SignalError> {
        use crate::data::retained_storage::{
            btree_structure_charge, RetainedStorageCharge as Charge,
        };
        let selections = producers
            .checked_add(transitions)
            .ok_or_else(|| SignalError::invalid_input("cause selection overflow"))?;
        let cause_owners = producers
            .checked_add(direct_consumers)
            .ok_or_else(|| SignalError::invalid_input("cause owner count overflow"))?;
        let projection_count = cause_owners
            .checked_add(waiter_projections)
            .ok_or_else(|| SignalError::invalid_input("cause projection count overflow"))?;
        btree_structure_charge::<NodeId, PendingRevalidationNodeProjection>(projection_count)
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<
                    (
                        crate::data::proof::invalidation::binding::OutputCommitOrdinal,
                        NodeId,
                    ),
                    bool,
                >(transitions)?)
            })
            .and_then(|charge| charge.checked_add(btree_structure_charge::<u32, ()>(cause_owners)?))
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<u32, ()>(direct_consumers)?)
            })
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<NodeId, usize>(direct_consumers)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PreparedCauseNodeReplacement>(
                    direct_consumers,
                )?)
            })
            .and_then(|charge| charge.checked_add(cause_row_backings(direct_consumers)?))
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PreparedConsumerCauseSet>(transitions)?)
            })
            .and_then(|charge| charge.checked_add(Charge::capacity::<NodeId>(producers)?))
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<ProducedAspectDelta>(producers)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<EpochCauseStoreEdit>(selections)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PendingCauseSetId>(selections)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<(PendingCauseSetId, NormalizedCauseSet)>(
                    direct_consumers,
                )?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<
                    crate::data::graph::storage::invalidation_causes::CanonicalCauseSetStore,
                >(1)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PreparedEpochCausePublication>(1)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<PreparedEpochDirectCausePublication>(1)?)
            })
            .map(Charge::bytes)
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)
    }

    pub(crate) fn prepare_epoch_direct_cause_publication(
        &self,
        admission: PreparedEpochDirectCauseAdmission,
        heads: &[EpochCauseHead],
        topology: &PreparedDependencyTopologyEpoch,
        work: &mut Work,
        mut preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
    ) -> Result<PreparedEpochDirectCausePublication, SignalError> {
        if heads
            .iter()
            .filter_map(|head| head.delta_ordinal)
            .ne(admission
                .commits
                .iter()
                .map(|commit| commit.output_commit_ordinal))
        {
            return Err(SignalError::internal("epoch cause heads lost commit order"));
        }
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<NodeId>(heads.len())?;
        }
        let mut clean_producers = Vec::with_capacity(heads.len());
        for head in heads {
            if head.clean {
                clean_producers.push(head.producer);
            }
        }
        for replacement in &admission.replacements {
            self.validate_epoch_pending_causes(
                replacement.consumer,
                &replacement.causes,
                topology,
                &admission.commits,
                &mut EvaluationWork::Conditional(work),
            )?;
        }
        let mut projections = BTreeMap::new();
        for update in topology.node_updates() {
            let Some(pending) = update.pending() else {
                continue;
            };
            let mut projected = PendingRevalidationNodeProjection::capture(self, update.node, work)
                .map_err(|denial| denial.into_signal_error())?;
            projected.pending = Some(pending.clone());
            if projected.state == NodeState::Clean {
                projected.state = NodeState::MaybeStale;
            }
            projected.has_pending_causes = false;
            projections.insert(update.node, projected);
        }
        for &producer in &clean_producers {
            projections.insert(
                producer,
                PendingRevalidationNodeProjection {
                    state: NodeState::Clean,
                    pending: None,
                    has_pending_causes: false,
                    dirty_aspects: AspectMask::EMPTY,
                    has_direct_basis: false,
                },
            );
        }
        // The canonical row buffer is already owned. Keep admitted rows in
        // its prefix; rejected rows are discarded after the projection pass.
        let mut accepted = admission.replacements;
        let mut accepted_len = 0usize;
        let mut suppressed = 0_u64;
        for index in 0..accepted.len() {
            let replacement = &accepted[index];
            let projected = match projections.entry(replacement.consumer) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::btree_map::Entry::Vacant(entry) => entry.insert(
                    PendingRevalidationNodeProjection::capture(self, replacement.consumer, work)
                        .map_err(|denial| denial.into_signal_error())?,
                ),
            };
            if projected.has_direct_basis {
                continue;
            }
            if replacement.causes.is_empty() {
                suppressed += 1;
            }
            projected.has_pending_causes = !replacement.causes.is_empty();
            projected.dirty_aspects = AspectMask::EMPTY;
            for cause in replacement.causes.iter() {
                projected.dirty_aspects.insert(cause.key.aspect);
            }
            projected.state = if projected.has_pending_causes {
                NodeState::Dirty
            } else {
                NodeState::MaybeStale
            };
            accepted.swap(accepted_len, index);
            accepted_len += 1;
        }
        accepted.truncate(accepted_len);
        let waiter_resolution = self
            .prepare_pending_revalidation_resolution_with_buckets(
                &clean_producers,
                projections,
                topology.prepared_waiter_buckets(),
                work,
            )
            .map_err(|denial| denial.into_signal_error())?;
        let mut slot_work = EvaluationWork::Conditional(work);
        self.admit_pending_cause_handle_reads(
            clean_producers.len() + admission.transitions.len(),
            &mut slot_work,
        )?;
        slot_work.reserve(
            admission
                .transitions
                .len()
                .checked_mul(std::mem::size_of::<PreparedCauseSlot>() + 1),
        )?;
        if let Some(budget) = preparation {
            budget.claim_vec::<EpochCauseStoreEdit>(heads.len() + admission.transitions.len())?;
            budget.claim_vec::<PreparedCauseNodeReplacement>(accepted.len())?;
            budget.claim_vec::<(PendingCauseSetId, NormalizedCauseSet)>(accepted.len())?;
            budget.claim_vec::<PendingCauseSetId>(heads.len() + admission.transitions.len())?;
            let claimed = crate::data::retained_storage::btree_structure_charge::<u32, ()>(
                heads.len() + accepted.len(),
            )
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            let allocated =
                crate::data::retained_storage::btree_structure_charge::<u32, ()>(accepted.len())
                    .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            budget.claim(claimed.bytes())?;
            budget.claim(allocated.bytes())?;
        }
        let mut cursor = self.cause_sets.prepare_cause_slots()?;
        let mut node_replacements = Vec::with_capacity(accepted.len());
        let mut finals = Vec::with_capacity(accepted.len());
        let mut edits = Vec::with_capacity(heads.len() + admission.transitions.len());
        let mut transitions = admission.transitions.into_iter().peekable();
        for head in heads {
            if head.clean {
                let release = self.node_pending_cause_set_id(head.producer)?;
                cursor.release(release, &mut slot_work)?;
                edits.push(EpochCauseStoreEdit::Release(release));
            }
            while transitions.peek().is_some_and(|((ordinal, _), _)| {
                Some(*ordinal) == head.delta_ordinal && head.delta_ordinal.is_some()
            }) {
                let ((_, consumer), empty) = transitions.next().expect("peeked transition");
                let Ok(index) = accepted.binary_search_by_key(&consumer, |entry| entry.consumer)
                else {
                    continue;
                };
                let entry = &mut accepted[index];
                let slot = if let Some(current) = entry.epoch_handle {
                    cursor.epoch_replacement_after(current, empty, &mut slot_work)?
                } else {
                    cursor.replacement(
                        self.node_pending_cause_set_id(consumer)?,
                        empty,
                        &mut slot_work,
                    )?
                };
                entry.epoch_handle = Some(slot.handle());
                edits.push(EpochCauseStoreEdit::Transition(slot));
            }
        }
        if transitions.next().is_some() {
            return Err(SignalError::internal(
                "epoch cause event has no producer head",
            ));
        }
        for replacement in accepted {
            let handle = replacement.epoch_handle.ok_or_else(|| {
                SignalError::internal("accepted epoch cause has no slot transition")
            })?;
            node_replacements.push(PreparedCauseNodeReplacement {
                consumer: replacement.consumer,
                cause_set: handle,
                cache: replacement.cache,
            });
            if !replacement.causes.is_empty() {
                finals.push((handle, replacement.causes));
            }
        }
        Ok(PreparedEpochDirectCausePublication {
            nodes: PreparedDirectCauseNodes {
                replacements: node_replacements,
                waiters: waiter_resolution,
            },
            stores: PreparedEpochDirectCauseStores {
                commits: admission.commits,
                edits,
                finals,
                counters: admission.counter_deltas,
            },
            suppressed_downstream: suppressed,
        })
    }
}

/// Epoch admission grows one canonical row Vec geometrically. Earlier
/// backings coexist during each realloc and are charged by the request budget.
fn cause_row_backings(
    count: usize,
) -> Result<
    crate::data::retained_storage::RetainedStorageCharge,
    crate::data::retained_storage::RetainedStoragePreparationDenial,
> {
    use crate::data::retained_storage::RetainedStorageCharge as Charge;
    let mut capacity = 0usize;
    let mut charge = Charge::ZERO;
    while capacity < count {
        capacity = capacity.saturating_mul(2).max(4);
        charge = charge.checked_add(Charge::capacity::<PreparedConsumerCauseSet>(capacity)?)?;
    }
    Ok(charge)
}

impl PreparedEpochDirectCausePublication {
    pub(crate) fn split_node_changes(
        self,
    ) -> (PreparedDirectCauseNodes, PreparedEpochDirectCauseStores) {
        (self.nodes, self.stores)
    }

    pub(crate) fn suppressed_downstream_count(&self) -> u64 {
        self.suppressed_downstream
    }
}

impl PreparedEpochDirectCauseStores {
    pub(crate) fn prepare_store(
        self,
        graph: &mut SignalGraph,
        work: &mut Work,
        preparation: Option<&mut crate::data::request_preparation::SignalPreparationBudget>,
    ) -> Result<PreparedEpochCausePublication, SignalError> {
        Ok(PreparedEpochCausePublication {
            store: graph.prepare_epoch_cause_store(
                self.edits,
                self.finals,
                self.commits,
                work,
                preparation,
            )?,
            counters: self.counters,
        })
    }
}

impl PreparedEpochCausePublication {
    pub(crate) fn publish(self, graph: &mut SignalGraph) {
        self.store.publish(graph);
        self.counters.publish(graph);
    }
}
