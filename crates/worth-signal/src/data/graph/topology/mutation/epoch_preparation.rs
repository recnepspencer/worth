//! Proposed dependency and reverse-membership storage for one graph epoch.
use worth_execution::MapKernelContext;

use super::super::PreparedReverseSubscriptionReplacement;
use super::epoch_subscribers::PreparedEpochSubscribers;
use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::storage::{
    DependencySetId, PreparedSegmentBatchInsertion, SubscriberSetId,
};
use crate::data::graph::{PreparedPendingRevalidationIndex, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::output::PreparedPartitionInternerExpansion;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, PendingDependencyRevalidation,
};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};

pub(crate) struct PreparedDependencyTopologyEpoch {
    desired: Vec<(NodeId, Vec<DependencyEdge>)>,
    interner: PreparedPartitionInternerExpansion,
    edge_sets: PreparedSegmentBatchInsertion<DependencyEdge, DependencySetId>,
    subscribers: PreparedEpochSubscribers,
    reverse: Vec<PreparedReverseSubscriptionReplacement>,
    node_updates: Vec<EpochTopologyNodeUpdate>,
    waiters: PreparedPendingRevalidationIndex,
}

pub(crate) struct PreparedDependencyTopologyStorage {
    interner: PreparedPartitionInternerExpansion,
    edge_sets: PreparedSegmentBatchInsertion<DependencyEdge, DependencySetId>,
    subscribers: PreparedEpochSubscribers,
    reverse: Vec<PreparedReverseSubscriptionReplacement>,
}

pub(crate) struct EpochTopologyNodeUpdate {
    pub(crate) node: NodeId,
    pub(crate) dependencies: Option<(DependencySetId, PendingDependencyRevalidation)>,
    pub(crate) subscribers: Option<SubscriberSetId>,
}

impl EpochTopologyNodeUpdate {
    pub(in crate::data::graph) fn apply_consumer(
        self,
        target: &mut crate::data::graph::storage::ConsumerNodeMutation<'_>,
    ) {
        if let Some((dependencies, pending)) = self.dependencies {
            target.replace_dependency_topology(dependencies, pending);
        }
        if let Some(subscribers) = self.subscribers {
            target.replace_subscriber_set(subscribers);
        }
    }

    pub(in crate::data::graph) fn apply(
        self,
        target: &mut crate::data::graph::storage::NodeEvaluationMutation<'_>,
    ) {
        if let Some((dependencies, pending)) = self.dependencies {
            target.replace_dependency_topology(dependencies, pending);
        }
        if let Some(subscribers) = self.subscribers {
            target.replace_subscriber_set(subscribers);
        }
    }

    pub(crate) fn pending(&self) -> Option<&PendingDependencyRevalidation> {
        self.dependencies.as_ref().map(|(_, pending)| pending)
    }
}

impl SignalGraph {
    pub(crate) fn prepare_dependency_topology_epoch(
        &mut self,
        reconciliations: &[(NodeId, &[DependencyEdge])],
        mut work: Option<&mut MapKernelContext<'_, '_>>,
        mut preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedDependencyTopologyEpoch, SignalError> {
        let total_edges = reconciliations
            .iter()
            .try_fold(0usize, |count, (_, edges)| count.checked_add(edges.len()))
            .ok_or_else(|| SignalError::invalid_input("epoch topology edge count overflow"))?;
        checkpoint(work.as_deref_mut(), reconciliations.len())?;
        let mut previous_edges = 0usize;
        for (node, _) in reconciliations {
            previous_edges = previous_edges
                .checked_add(self.raw_dependencies_of(*node)?.len())
                .ok_or_else(|| SignalError::invalid_input("epoch previous edge count overflow"))?;
        }
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<(NodeId, Vec<DependencyEdge>)>(reconciliations.len())?;
            budget.claim_vec::<PreparedReverseSubscriptionReplacement>(reconciliations.len())?;
            budget.claim_vec::<Vec<DependencyEdge>>(reconciliations.len())?;
            let selected = reconciliations
                .len()
                .checked_add(total_edges)
                .and_then(|n| n.checked_add(previous_edges))
                .ok_or_else(|| SignalError::invalid_input("epoch topology node count overflow"))?;
            budget.claim_vec::<EpochTopologyNodeUpdate>(selected)?;
            budget.claim_vec::<u8>(selected.saturating_mul(128))?;
            budget.claim_vec::<(NodeId, Vec<NodeId>, Vec<NodeId>)>(reconciliations.len())?;
            for (_, edges) in reconciliations {
                budget.claim_vec::<DependencyEdge>(edges.len())?;
                budget.claim_vec::<DependencyEdge>(edges.len())?;
                budget.claim_vec::<DependencyEdge>(edges.len())?;
                for edge in *edges {
                    if let Some(scope) = edge.scope_ref() {
                        let path = scope.path();
                        let bytes = path
                            .depth()
                            .checked_mul(std::mem::size_of::<String>())
                            .and_then(|base| base.checked_add(path.checked_segment_bytes()?))
                            .and_then(|bytes| u64::try_from(bytes).ok())
                            .ok_or_else(|| {
                                SignalError::invalid_input("topology scope clone size overflow")
                            })?;
                        for _ in 0..3 {
                            budget.claim(bytes)?;
                        }
                    }
                }
            }
        }
        let canonical = super::preflight::canonicalize_and_preflight_with_work(
            self,
            reconciliations,
            work.as_deref_mut(),
        )?;
        let intern_work = canonical
            .iter()
            .flat_map(|(_, edges)| edges.as_slice())
            .filter_map(DependencyEdge::scope_ref)
            .try_fold(0usize, |count, scope| {
                count
                    .checked_add(scope.path().depth())?
                    .checked_add(scope.path().checked_segment_bytes()?)
            })
            .ok_or_else(|| SignalError::invalid_input("epoch interner work overflow"))?;
        checkpoint(work.as_deref_mut(), intern_work)?;
        let interner = self
            .observation
            .partition_interner
            .prepare_expansion_for_subscriptions(canonical.iter().flat_map(|(_, edges)| {
                edges
                    .as_slice()
                    .iter()
                    .filter_map(DependencyEdge::scope_ref)
            }))?;
        let mut desired = Vec::with_capacity(canonical.len());
        let mut reverse = Vec::with_capacity(canonical.len());
        for (node, edges) in canonical {
            checkpoint(work.as_deref_mut(), edges.as_slice().len())?;
            let interned = edges
                .as_slice()
                .iter()
                .map(|edge| {
                    let Some(scope) = edge.scope_ref() else {
                        return DependencyEdge::new(edge.source(), edge.aspect());
                    };
                    DependencyEdge::with_scope(
                        edge.source(),
                        edge.aspect(),
                        scope.clone(),
                        interner.interned(&self.observation.partition_interner, scope),
                    )
                })
                .collect::<Vec<_>>();
            let replacement = if let Some(request) = work.as_deref_mut() {
                let mut admit = |units: usize| {
                    let units = u64::try_from(units).map_err(|_| {
                        SignalError::invalid_input("reverse subscription work overflow")
                    })?;
                    request.checkpoint(units).map_err(|_| {
                        SignalError::invalid_input("reverse subscription work stopped")
                    })
                };
                self.prepare_reverse_subscription_replacement(
                    node,
                    &interned,
                    &mut crate::logic::evaluation::EvaluationWork::RequestCheckpoint(&mut admit),
                )?
            } else {
                self.prepare_reverse_subscription_replacement(
                    node,
                    &interned,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary,
                )?
            };
            reverse.push(replacement);
            desired.push((node, interned));
        }
        let batches = desired
            .iter()
            .map(|(_, edges)| edges.clone())
            .collect::<Vec<_>>();
        checkpoint(
            work.as_deref_mut(),
            total_edges.saturating_mul(desired.len().saturating_add(1)),
        )?;
        let edge_sets = with_segment_work(work.as_deref_mut(), |observed| {
            self.topology.dependency_edges.prepare_batch_insertion(
                &batches,
                observed,
                preparation.as_deref_mut(),
            )
        })?;
        let subscribers = self.prepare_epoch_subscribers(
            &desired,
            work.as_deref_mut(),
            preparation.as_deref_mut(),
        )?;
        let mut node_updates = std::collections::BTreeMap::<NodeId, EpochTopologyNodeUpdate>::new();
        let mut waiter_changes = Vec::with_capacity(reconciliations.len());
        for ((node, edges), &dependencies) in desired.iter().zip(edge_sets.ids()) {
            checkpoint(work.as_deref_mut(), edges.len().saturating_add(1))?;
            if self.raw_dependencies_of(*node)? == edges.as_slice() {
                continue;
            }
            let revision = DependencyRevision(
                self.node_dependency_revision(*node)?
                    .0
                    .checked_add(1)
                    .ok_or_else(|| SignalError::internal("dependency revision overflow"))?,
            );
            let previous_len = self
                .node_pending_revalidation(*node)?
                .map_or(0, |pending| pending.unresolved_producers().len());
            if let Some(budget) = preparation.as_deref_mut() {
                budget.claim_vec::<NodeId>(previous_len)?;
                budget.claim_vec::<NodeId>(edges.len())?;
                budget.claim_vec::<NodeId>(edges.len())?;
            }
            let previous = self
                .node_pending_revalidation(*node)?
                .map(|pending| pending.unresolved_producers().to_vec())
                .unwrap_or_default();
            let mut current = Vec::with_capacity(edges.len());
            current.extend(edges.iter().filter_map(|edge| {
                (!matches!(self.get_state(edge.source()), Ok(NodeState::Clean)))
                    .then_some(edge.source())
            }));
            waiter_changes.push((*node, previous, current.clone()));
            node_updates.insert(
                *node,
                EpochTopologyNodeUpdate {
                    node: *node,
                    dependencies: Some((
                        dependencies,
                        PendingDependencyRevalidation::structural(revision, current),
                    )),
                    subscribers: None,
                },
            );
        }
        for &(source, subscribers_id) in &subscribers.updates {
            checkpoint(work.as_deref_mut(), 1)?;
            let update = node_updates
                .entry(source)
                .or_insert_with(|| EpochTopologyNodeUpdate {
                    node: source,
                    dependencies: None,
                    subscribers: None,
                });
            update.subscribers = Some(subscribers_id);
        }
        let waiters = PreparedPendingRevalidationIndex::for_replacements(self, &waiter_changes);
        Ok(PreparedDependencyTopologyEpoch {
            desired,
            interner,
            edge_sets,
            reverse,
            node_updates: node_updates.into_values().collect(),
            subscribers,
            waiters,
        })
    }
}

fn checkpoint(
    work: Option<&mut MapKernelContext<'_, '_>>,
    units: usize,
) -> Result<(), SignalError> {
    if let Some(work) = work {
        work.checkpoint(
            u64::try_from(units)
                .map_err(|_| SignalError::invalid_input("epoch topology work overflow"))?,
        )
        .map_err(|_| SignalError::invalid_input("epoch topology work stopped"))?;
    }
    Ok(())
}

pub(super) fn with_segment_work<R>(
    request: Option<&mut MapKernelContext<'_, '_>>,
    operation: impl FnOnce(&mut Work<'_>) -> Result<R, SignalError>,
) -> Result<R, SignalError> {
    let mut work = Work::new(usize::MAX);
    let Some(request) = request else {
        return operation(&mut work);
    };
    let mut checkpoint = |units: usize| {
        let units =
            u64::try_from(units).map_err(|_| RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits: usize::MAX,
            })?;
        request
            .checkpoint(units)
            .map_err(|_| RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits: usize::MAX,
            })
    };
    let mut observed = work.reborrow_with_checkpoint(&mut checkpoint);
    operation(&mut observed)
}

impl PreparedDependencyTopologyEpoch {
    pub(crate) fn proposed_edges(&self, node: NodeId) -> Option<&[DependencyEdge]> {
        self.desired
            .binary_search_by_key(&node, |(target, _)| *target)
            .ok()
            .map(|index| self.desired[index].1.as_slice())
    }

    pub(crate) fn node_updates(&self) -> &[EpochTopologyNodeUpdate] {
        &self.node_updates
    }

    pub(crate) fn prepared_waiter_buckets(
        &self,
    ) -> std::collections::BTreeMap<NodeId, im::OrdSet<NodeId>> {
        self.waiters.cloned_buckets()
    }

    pub(crate) fn into_publication_parts(
        self,
    ) -> (
        PreparedDependencyTopologyStorage,
        Vec<EpochTopologyNodeUpdate>,
        PreparedPendingRevalidationIndex,
    ) {
        (
            PreparedDependencyTopologyStorage {
                interner: self.interner,
                edge_sets: self.edge_sets,
                subscribers: self.subscribers,
                reverse: self.reverse,
            },
            self.node_updates,
            self.waiters,
        )
    }
}

impl PreparedDependencyTopologyStorage {
    pub(crate) fn publish(self, graph: &mut SignalGraph) {
        graph
            .observation
            .partition_interner
            .publish_prepared_expansion(self.interner);
        graph
            .topology
            .dependency_edges
            .publish_batch_insertion(self.edge_sets);
        self.subscribers.publish(graph);
        for replacement in self.reverse {
            replacement.publish(graph);
        }
    }
}
