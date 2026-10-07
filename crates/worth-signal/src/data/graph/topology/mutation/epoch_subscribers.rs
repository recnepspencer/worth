//! Cumulative authoritative subscriber sets for a dependency rewrite epoch.
use std::collections::{BTreeMap, BTreeSet};

use worth_execution::MapKernelContext;

use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::storage::{PreparedSegmentBatchInsertion, SubscriberSetId};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::request_preparation::SignalPreparationBudget;

pub(super) struct PreparedEpochSubscribers {
    sets: PreparedSegmentBatchInsertion<NodeId, SubscriberSetId>,
    pub(super) updates: Vec<(NodeId, SubscriberSetId)>,
}

impl SignalGraph {
    /// One touched source's subscriber-set seed, canonical replacement and
    /// segment insertion ceiling. Both old and proposed edges can touch it.
    pub(crate) fn epoch_subscriber_source_capacity_bound(
        &self,
        source: NodeId,
        maximum_additions: usize,
    ) -> Result<u64, SignalError> {
        use crate::data::retained_storage::{
            btree_structure_charge, RetainedStorageCharge as Charge,
        };
        let count = self
            .raw_subscribers_of(source)?
            .len()
            .checked_add(maximum_additions)
            .ok_or_else(|| SignalError::invalid_input("subscriber fanout overflow"))?;
        let structure = btree_structure_charge::<NodeId, ()>(count)
            .and_then(|charge| {
                charge.checked_add(btree_structure_charge::<
                    NodeId,
                    std::collections::BTreeSet<NodeId>,
                >(1)?)
            })
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<NodeId>(count.checked_mul(3).ok_or(
                    crate::data::retained_storage::RetainedStoragePreparationDenial::ChargeOverflow,
                )?)?)
            })
            .and_then(|charge| charge.checked_add(Charge::capacity::<Vec<NodeId>>(2)?))
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<(NodeId, SubscriberSetId)>(1)?)
            })
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        structure
            .checked_add(
                self.topology
                    .subscriber_edges
                    .insertion_payload_capacity_bound(count, 0)?,
            )
            .map(Charge::bytes)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    pub(crate) fn epoch_subscriber_batch_structure_capacity_bound(
        &self,
        source_count: usize,
    ) -> Result<u64, SignalError> {
        Ok(self
            .topology
            .subscriber_edges
            .batch_structure_capacity_bound(source_count)?
            .bytes())
    }

    pub(super) fn prepare_epoch_subscribers(
        &mut self,
        desired: &[(NodeId, Vec<DependencyEdge>)],
        mut work: Option<&mut MapKernelContext<'_, '_>>,
        mut preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedEpochSubscribers, SignalError> {
        let mut changed = BTreeMap::<NodeId, BTreeSet<NodeId>>::new();
        for (consumer, edges) in desired {
            let current = self.raw_dependencies_of(*consumer)?;
            if current == edges.as_slice() {
                continue;
            }
            checkpoint(
                work.as_deref_mut(),
                current.len().saturating_add(edges.len()),
            )?;
            if let Some(budget) = preparation.as_deref_mut() {
                let touched = current.len().checked_add(edges.len()).ok_or_else(|| {
                    SignalError::invalid_input("subscriber source count overflow")
                })?;
                budget.claim_vec::<u8>(touched.saturating_mul(128))?;
            }
            let touched_count = current.len().saturating_add(edges.len());
            checkpoint(
                work.as_deref_mut(),
                touched_count
                    .saturating_mul(touched_count.checked_ilog2().unwrap_or(0) as usize + 2),
            )?;
            let touched = current
                .iter()
                .chain(edges)
                .map(DependencyEdge::source)
                .collect::<BTreeSet<_>>();
            for source in touched {
                if !changed.contains_key(&source) {
                    if let Some(budget) = preparation.as_deref_mut() {
                        budget.claim_vec::<u8>(128)?;
                    }
                }
                if let std::collections::btree_map::Entry::Vacant(entry) = changed.entry(source) {
                    let subscribers = self.raw_subscribers_of(source)?;
                    checkpoint(
                        work.as_deref_mut(),
                        subscribers.len().saturating_mul(
                            subscribers.len().checked_ilog2().unwrap_or(0) as usize + 2,
                        ),
                    )?;
                    if let Some(budget) = preparation.as_deref_mut() {
                        budget.claim_vec::<NodeId>(subscribers.len())?;
                        budget.claim_vec::<u8>(subscribers.len().saturating_mul(128))?;
                    }
                    entry.insert(subscribers.iter().copied().collect());
                }
                let members = changed.get_mut(&source).expect("seeded source set");
                checkpoint(
                    work.as_deref_mut(),
                    edges
                        .len()
                        .saturating_add(members.len().checked_ilog2().unwrap_or(0) as usize + 2),
                )?;
                if edges.iter().any(|edge| edge.source() == source) {
                    if let Some(budget) = preparation.as_deref_mut() {
                        budget.claim_vec::<u8>(128)?;
                    }
                    members.insert(*consumer);
                } else {
                    members.remove(consumer);
                }
            }
        }
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<Vec<NodeId>>(changed.len())?;
            budget.claim_vec::<Vec<NodeId>>(changed.len())?;
            budget.claim_vec::<NodeId>(changed.len())?;
            budget.claim_vec::<(NodeId, SubscriberSetId)>(changed.len())?;
            budget.claim_vec::<SubscriberSetId>(changed.len())?;
        }
        let mut sources = Vec::with_capacity(changed.len());
        let mut batches = Vec::with_capacity(changed.len());
        for (source, subscribers) in changed {
            checkpoint(work.as_deref_mut(), subscribers.len())?;
            if let Some(budget) = preparation.as_deref_mut() {
                // The selected segment store may own one further copy.
                budget.claim_vec::<NodeId>(subscribers.len())?;
                budget.claim_vec::<NodeId>(subscribers.len())?;
            }
            sources.push(source);
            batches.push(subscribers.into_iter().collect());
        }
        // All source membership changes are one canonical reducer draft. The
        // worker footprint describes proposals by target; only this owner
        // selects shared source handles and publishes them once after join.
        let sets = super::epoch_preparation::with_segment_work(work, |observed| {
            self.topology
                .subscriber_edges
                .prepare_batch_insertion(&batches, observed, preparation)
        })?;
        let updates = sources
            .into_iter()
            .zip(sets.ids().iter().copied())
            .collect();
        Ok(PreparedEpochSubscribers { sets, updates })
    }
}

impl PreparedEpochSubscribers {
    pub(super) fn publish(self, graph: &mut SignalGraph) {
        graph
            .topology
            .subscriber_edges
            .publish_batch_insertion(self.sets);
    }
}

fn checkpoint(
    work: Option<&mut MapKernelContext<'_, '_>>,
    units: usize,
) -> Result<(), SignalError> {
    if let Some(work) = work {
        work.checkpoint(
            u64::try_from(units)
                .map_err(|_| SignalError::invalid_input("subscriber preparation work overflow"))?,
        )
        .map_err(|_| SignalError::invalid_input("subscriber preparation work stopped"))?;
    }
    Ok(())
}
