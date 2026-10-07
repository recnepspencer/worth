//! One node evaluation edit for topology, outputs, causes, and waiter resolution.
use std::collections::BTreeMap;

use super::super::{EffectStateMutation, PreparedEffectArtifactWrite, PreparedEffectNodeState};
use super::{ApplyCommitPacket, SignalError, SignalGraph};
use crate::data::dependency::DependencySnapshotId;
use crate::data::graph::runtime::graph::PreparedDirectCauseNodes;
use crate::data::graph::runtime::graph::{
    map_node_edit as map_edit, OperationalNodePayload, PreparedRetainedNodeEdit,
    RetainedNodeEditOutcome, RetainedNodeEditPreparation, RetainedNodePayload, SelectedNodeDraft,
    SelectedNodeRole,
};
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::storage::{
    ConsumerNodeMutation, NodeEvaluationMutation, PreparedInvalidationCache,
};
use crate::data::graph::topology::EpochTopologyNodeUpdate;
use crate::data::graph::PreparedPendingRevalidationIndex;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
};

mod capacity;
mod node_mutation;
mod physical_roots;
#[cfg(all(test, feature = "test-peak-allocation"))]
mod physical_roots_tests;
mod publication;
mod semantic_order;

pub(super) struct EpochProducerNodeChange {
    pub(super) order: usize,
    pub(super) semantic_seed: crate::data::graph::EpochSemanticSeed,
    pub(super) apply: ApplyCommitPacket,
    pub(super) state: PreparedEffectNodeState,
    pub(super) write: PreparedEffectArtifactWrite,
    pub(super) snapshot: Option<DependencySnapshotId>,
    pub(super) delta: Option<ProducedAspectDelta>,
}

pub(super) struct EpochPublishedProducer {
    pub(super) order: usize,
    pub(super) semantic_seed: Option<crate::data::graph::EpochSemanticSeed>,
    pub(super) apply: ApplyCommitPacket,
    pub(super) mutation: EffectStateMutation,
    pub(super) causality_changed: bool,
    pub(super) runtime_write: bool,
    pub(super) delta: Option<ProducedAspectDelta>,
    pub(super) semantic_artifacts: Option<crate::data::graph::PreparedSemanticArtifactImage>,
}

#[derive(Default)]
struct EpochNodeChange {
    topology: Option<EpochTopologyNodeUpdate>,
    producer_index: Option<usize>,
    cause: Option<(PendingCauseSetId, PreparedInvalidationCache)>,
    projected: Option<crate::data::graph::PendingRevalidationNodeProjection>,
}

pub(super) struct PreparedEpochNodeEdits {
    ordinary: Option<PreparedOrdinaryEpochNodeEdits>,
    retained: Option<PreparedRetainedNodeEdit<Vec<EpochPublishedProducer>>>,
    waiters: PreparedPendingRevalidationIndex,
}

struct PreparedOrdinaryEpochNodeEdits {
    selected: Vec<(usize, SelectedNodeDraft)>,
    published: Vec<EpochPublishedProducer>,
    node_count: usize,
}

impl SignalGraph {
    pub(super) fn prepare_epoch_node_edits(
        &self,
        topology: Vec<EpochTopologyNodeUpdate>,
        mut producers: Vec<Option<EpochProducerNodeChange>>,
        direct: PreparedDirectCauseNodes,
        work: &mut Work,
        mut preparation: Option<&mut SignalPreparationBudget>,
        mut prepare_semantic: impl FnMut(
            &mut RetainedNodePayload,
            &mut EpochPublishedProducer,
            &mut Work,
            Option<&mut SignalPreparationBudget>,
        ) -> Result<(), SignalError>,
    ) -> Result<PreparedEpochNodeEdits, SignalError> {
        let producer_width = producers.len();
        if let Some(budget) = preparation.as_deref_mut() {
            let selected = topology
                .len()
                .checked_add(producers.len())
                .and_then(|n| n.checked_add(direct.selected_nodes().len()))
                .ok_or_else(|| SignalError::invalid_input("epoch node selection overflow"))?;
            budget.claim_vec::<usize>(selected)?;
            budget.claim_vec::<EpochPublishedProducer>(producers.len())?;
            budget.claim_vec::<semantic_order::PlanSemanticSlot>(producers.len())?;
        }
        work.reserve_visits(producer_width)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
        let mut semantic_slots = vec![None; producer_width];
        let mut nodes = BTreeMap::<NodeId, EpochNodeChange>::new();
        for update in topology {
            let node = update.node;
            node_change(&mut nodes, node, work, preparation.as_deref_mut())?.topology =
                Some(update);
        }
        for (index, producer) in producers.iter().enumerate() {
            let node = producer
                .as_ref()
                .expect("new epoch producer")
                .apply
                .effect
                .operational
                .node;
            if node_change(&mut nodes, node, work, preparation.as_deref_mut())?
                .producer_index
                .replace(index)
                .is_some()
            {
                return Err(SignalError::internal("duplicate epoch producer"));
            }
        }
        let waiters = direct.visit_node_changes(|node, cause, projected| {
            let change = node_change(&mut nodes, node, work, preparation.as_deref_mut())?;
            change.cause = cause;
            change.projected = Some(projected);
            Ok(())
        })?;
        if nodes.is_empty() {
            return Err(SignalError::internal("empty epoch node edit"));
        }
        for &node in nodes.keys() {
            self.get_state(node)?;
        }
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<(usize, SelectedNodeDraft)>(nodes.len())?;
            budget.claim_vec::<(usize, SelectedNodeRole)>(nodes.len())?;
            for (&node, change) in &nodes {
                let bytes = if change.producer_index.is_some() {
                    self.epoch_node_clone_charge(node, work)?
                } else {
                    self.epoch_consumer_clone_charge(node, work)?
                };
                budget.claim(bytes.bytes())?;
            }
        }
        if let Some(ledger) = self.arena.retained_node_ledger.as_ref() {
            work.reserve_visits(nodes.len())
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let indices = nodes
                .iter()
                .map(|(node, change)| {
                    (
                        node.index() as usize,
                        if change.producer_index.is_some() {
                            SelectedNodeRole::ProducerFull
                        } else {
                            SelectedNodeRole::ConsumerOperational
                        },
                    )
                })
                .collect::<Vec<_>>();
            let maximum = usize::try_from(
                self.installed_runtime_policy()
                    .conditional_evaluation_budget()
                    .maximum_retained_bytes,
            )
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            let maximum = Charge::capacity::<u8>(maximum)
                .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            let staged = self
                .arena
                .prepare_selected_epoch_nodes(ledger, &indices, maximum, work, |payloads, work| {
                    let mut published = Vec::with_capacity(producer_width);
                    for (payload_index, ((_, change), payload)) in
                        nodes.into_iter().zip(payloads.iter_mut()).enumerate()
                    {
                        let plan_index = change.producer_index;
                        let producer = plan_index
                            .map(|index| producers[index].take().expect("unique epoch producer"));
                        match payload {
                            SelectedNodeDraft::ProducerFull(payload) => {
                                node_mutation::admit_producer_version_write(
                                    self,
                                    producer.as_ref(),
                                    work,
                                )?;
                                let mut target = NodeEvaluationMutation::draft(
                                    &mut payload.hot,
                                    &mut payload.warm,
                                    &mut payload.cold,
                                );
                                let published_producer =
                                    change.apply(&mut target, producer).expect("producer draft");
                                semantic_order::record_slot(
                                    &mut semantic_slots,
                                    plan_index.expect("published producer has plan index"),
                                    payload_index,
                                    published.len(),
                                    work,
                                )?;
                                published.push(published_producer);
                            }
                            SelectedNodeDraft::ConsumerOperational(payload) => {
                                let mut target = ConsumerNodeMutation::draft(
                                    &mut payload.hot,
                                    &mut payload.warm,
                                );
                                change.apply_consumer(&mut target);
                            }
                        }
                    }
                    semantic_order::prepare_in_plan_order(
                        payloads,
                        &mut published,
                        semantic_slots,
                        work,
                        preparation.as_deref_mut(),
                        |payload| match payload {
                            SelectedNodeDraft::ProducerFull(full) => full,
                            SelectedNodeDraft::ConsumerOperational(_) => {
                                unreachable!("semantic producer role")
                            }
                        },
                        &mut prepare_semantic,
                    )?;
                    Ok::<_, SignalError>(published)
                })
                .map_err(map_edit)?;
            let retained = match staged {
                RetainedNodeEditPreparation::Ready(roots) => {
                    roots
                        .try_split_output(|result| result.map(|published| (published, ())))?
                        .0
                }
                RetainedNodeEditPreparation::Rejected { denial, .. } => {
                    return Err(map_edit(denial))
                }
            };
            Ok(PreparedEpochNodeEdits {
                ordinary: None,
                retained: Some(retained),
                waiters,
            })
        } else {
            let mut selected = Vec::with_capacity(nodes.len());
            for &node in nodes.keys() {
                let index = node.index() as usize;
                let mut clone_work = crate::logic::evaluation::EvaluationWork::Conditional(work);
                self.arena.warm[index].admit_clone_work(&mut clone_work)?;
                let hot = self.arena.hot[index]
                    .as_ref()
                    .expect("validated epoch node")
                    .clone();
                let payload = if nodes
                    .get(&node)
                    .expect("selected node")
                    .producer_index
                    .is_some()
                {
                    if let Some(cold) = self.arena.cold[index].as_ref() {
                        cold.admit_clone_work(&mut clone_work)?;
                    }
                    SelectedNodeDraft::ProducerFull(RetainedNodePayload {
                        hot,
                        warm: self.arena.warm[index].clone(),
                        cold: self.arena.cold[index].clone(),
                    })
                } else {
                    SelectedNodeDraft::ConsumerOperational(OperationalNodePayload {
                        hot,
                        warm: self.arena.warm[index].operational_consumer_draft(),
                    })
                };
                selected.push((index, payload));
            }
            let mut published = Vec::with_capacity(producer_width);
            for (payload_index, ((_, change), (_, payload))) in
                nodes.into_iter().zip(selected.iter_mut()).enumerate()
            {
                let plan_index = change.producer_index;
                let producer =
                    plan_index.map(|index| producers[index].take().expect("unique epoch producer"));
                match payload {
                    SelectedNodeDraft::ProducerFull(payload) => {
                        node_mutation::admit_producer_version_write(self, producer.as_ref(), work)?;
                        let mut target = NodeEvaluationMutation::draft(
                            &mut payload.hot,
                            &mut payload.warm,
                            &mut payload.cold,
                        );
                        let published_producer =
                            change.apply(&mut target, producer).expect("producer draft");
                        semantic_order::record_slot(
                            &mut semantic_slots,
                            plan_index.expect("published producer has plan index"),
                            payload_index,
                            published.len(),
                            work,
                        )?;
                        published.push(published_producer);
                    }
                    SelectedNodeDraft::ConsumerOperational(payload) => {
                        let mut target =
                            ConsumerNodeMutation::draft(&mut payload.hot, &mut payload.warm);
                        change.apply_consumer(&mut target);
                    }
                }
            }
            semantic_order::prepare_in_plan_order(
                &mut selected,
                &mut published,
                semantic_slots,
                work,
                preparation,
                |(_, payload)| match payload {
                    SelectedNodeDraft::ProducerFull(full) => full,
                    SelectedNodeDraft::ConsumerOperational(_) => {
                        unreachable!("semantic producer role")
                    }
                },
                &mut prepare_semantic,
            )?;
            self.admit_ordinary_epoch_node_writes(selected.len(), published.len(), work)?;
            Ok(PreparedEpochNodeEdits {
                ordinary: Some(PreparedOrdinaryEpochNodeEdits {
                    selected,
                    published,
                    node_count: self.arena.hot.len(),
                }),
                retained: None,
                waiters,
            })
        }
    }
}

fn node_change<'a>(
    nodes: &'a mut BTreeMap<NodeId, EpochNodeChange>,
    node: NodeId,
    work: &mut Work,
    budget: Option<&mut SignalPreparationBudget>,
) -> Result<&'a mut EpochNodeChange, SignalError> {
    use crate::data::retained_storage::{btree_structure_charge, ordered_lookup_steps};
    let lookup = ordered_lookup_steps(nodes.len()).saturating_mul(2);
    work.reserve_visits(lookup)
        .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
    if !nodes.contains_key(&node) {
        if let Some(budget) = budget {
            let before = btree_structure_charge::<NodeId, EpochNodeChange>(nodes.len())
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let next = nodes
                .len()
                .checked_add(1)
                .ok_or_else(|| SignalError::invalid_input("epoch node count overflow"))?;
            let after = btree_structure_charge::<NodeId, EpochNodeChange>(next)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            let growth = after
                .checked_sub(before)
                .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)?;
            if nodes.is_empty() {
                budget.claim(before.bytes())?;
            }
            budget.claim(growth.bytes())?;
        }
    }
    Ok(nodes.entry(node).or_default())
}
