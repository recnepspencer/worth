use std::collections::{BTreeMap, BTreeSet};

use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::transaction::runtime::transaction::{
    CommittedObservationEvent, CommittedObservationEventSummary,
};

use super::super::runtime_state::SignalRuntime;
use super::{
    MatchingObserverSet, ObservationHandle, ObservationListener, ObservationNotice,
    ObservationPolicy, ObservationReadContext, ObservationRegistrySummary, ObservedNodeSet,
    ObserverId,
};

struct ObserverRegistration<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    handle_id: super::ObservationHandleId,
    policy: ObservationPolicy,
    observed_nodes: ObservedNodeSet,
    listener: Box<dyn ObservationListener<D, I, E, Ctx, T>>,
}

pub struct RuntimeObservationRegistry<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    next_observer_id: u64,
    next_handle_id: u64,
    registrations: BTreeMap<ObserverId, ObserverRegistration<D, I, E, Ctx, T>>,
    observers_by_node: BTreeMap<NodeId, BTreeSet<ObserverId>>,
}

impl<D, I, E, Ctx, T> Default for RuntimeObservationRegistry<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn default() -> Self {
        Self {
            next_observer_id: 1,
            next_handle_id: 1,
            registrations: BTreeMap::new(),
            observers_by_node: BTreeMap::new(),
        }
    }
}

impl<D, I, E, Ctx, T> RuntimeObservationRegistry<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn register_nodes(
        &mut self,
        policy: ObservationPolicy,
        observed_nodes: ObservedNodeSet,
        listener: Box<dyn ObservationListener<D, I, E, Ctx, T>>,
    ) -> ObservationHandle {
        let observer_id = ObserverId(self.next_observer_id);
        self.next_observer_id += 1;
        let handle_id = super::ObservationHandleId(self.next_handle_id);
        self.next_handle_id += 1;

        for node in observed_nodes.iter() {
            self.observers_by_node
                .entry(node)
                .or_default()
                .insert(observer_id);
        }

        self.registrations.insert(
            observer_id,
            ObserverRegistration {
                handle_id,
                policy,
                observed_nodes,
                listener,
            },
        );

        ObservationHandle {
            observer_id,
            handle_id,
        }
    }

    pub fn unsubscribe(&mut self, handle: ObservationHandle) -> bool {
        let Some(existing) = self.registrations.get(&handle.observer_id()) else {
            return false;
        };
        if existing.handle_id != handle.handle_id() {
            return false;
        }
        let registration = self
            .registrations
            .remove(&handle.observer_id())
            .expect("observation registration should still exist");
        for node in registration.observed_nodes.iter() {
            let remove_entry = if let Some(set) = self.observers_by_node.get_mut(&node) {
                set.remove(&handle.observer_id());
                set.is_empty()
            } else {
                false
            };
            if remove_entry {
                self.observers_by_node.remove(&node);
            }
        }
        true
    }

    pub fn summary(&self) -> ObservationRegistrySummary {
        ObservationRegistrySummary {
            active_observer_count: self.registrations.len(),
            indexed_node_count: self.observers_by_node.len(),
            ordered_observer_ids: self.registrations.keys().copied().collect(),
        }
    }

    pub fn matching_observers_for_node(&self, node: NodeId) -> MatchingObserverSet {
        let observers = self
            .observers_by_node
            .get(&node)
            .map(|ids| ids.iter().copied().collect())
            .unwrap_or_default();
        MatchingObserverSet::new(observers)
    }

    /// Every node with at least one live observer, in node order.
    ///
    /// Cost: O(indexed nodes). `unsubscribe` removes index entries that
    /// become empty, so this never yields a node without observers.
    pub fn observed_node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.observers_by_node
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(node, _)| *node)
    }

    pub fn has_matching_observers_for_node(&self, node: NodeId) -> bool {
        self.observers_by_node
            .get(&node)
            .is_some_and(|ids| !ids.is_empty())
    }

    pub fn for_each_matching_observer_for_node<F>(&self, node: NodeId, mut visit: F)
    where
        F: FnMut(ObserverId),
    {
        let Some(observer_ids) = self.observers_by_node.get(&node) else {
            return;
        };
        for &observer_id in observer_ids {
            visit(observer_id);
        }
    }

    pub fn registration_count(&self) -> usize {
        self.registrations.len()
    }

    pub fn registration_for(
        &self,
        observer_id: ObserverId,
    ) -> Option<(
        super::ObservationHandleId,
        ObservationPolicy,
        &ObservedNodeSet,
    )> {
        self.registrations.get(&observer_id).map(|registration| {
            (
                registration.handle_id,
                registration.policy,
                &registration.observed_nodes,
            )
        })
    }

    pub fn notify_preview(
        &self,
        runtime: &SignalRuntime<D, I, E, Ctx, T>,
        observer_id: ObserverId,
    ) -> bool {
        let Some(registration) = self.registrations.get(&observer_id) else {
            return false;
        };
        let notice = ObservationNotice {
            observer_id,
            handle_id: registration.handle_id,
            policy: registration.policy,
            observed_nodes: &registration.observed_nodes,
            matched_nodes: &registration.observed_nodes,
            touched: false,
            recomputed: false,
            meaningful_change: false,
            trigger_matched: false,
        };
        registration
            .listener
            .on_observation(ObservationReadContext::new(runtime), &notice);
        true
    }

    pub(in crate::logic::transaction::runtime) fn deliver_committed(
        &self,
        graph: &SignalGraph,
        deliveries: &[CommittedObservationEvent],
    ) -> usize {
        let mut delivered = 0;
        for delivery in deliveries {
            let Some(registration) = self.registrations.get(&delivery.observer_id()) else {
                continue;
            };
            let ctx = ObservationReadContext::from_graph(graph);
            let notice = ObservationNotice {
                observer_id: delivery.observer_id(),
                handle_id: delivery.handle_id(),
                policy: delivery.policy(),
                observed_nodes: delivery.observed_nodes(),
                matched_nodes: delivery.matched_nodes(),
                touched: delivery.touched(),
                recomputed: delivery.recomputed(),
                meaningful_change: delivery.meaningful_change(),
                trigger_matched: delivery.trigger_matched(),
            };
            registration.listener.on_observation(ctx, &notice);
            delivered += 1;
        }
        delivered
    }

    pub(crate) fn deliver_boundary_summaries(
        &self,
        graph: &SignalGraph,
        deliveries: &[CommittedObservationEventSummary],
    ) -> usize {
        let mut delivered = 0;
        for delivery in deliveries {
            let Some(registration) = self.registrations.get(&delivery.observer_id) else {
                continue;
            };
            let ctx = ObservationReadContext::from_graph(graph);
            let notice = ObservationNotice {
                observer_id: delivery.observer_id,
                handle_id: delivery.handle_id,
                policy: delivery.policy,
                observed_nodes: &delivery.observed_nodes,
                matched_nodes: &delivery.matched_nodes,
                touched: delivery.touched,
                recomputed: delivery.recomputed,
                meaningful_change: delivery.meaningful_change,
                trigger_matched: delivery.trigger_matched,
            };
            registration.listener.on_observation(ctx, &notice);
            delivered += 1;
        }
        delivered
    }
}
