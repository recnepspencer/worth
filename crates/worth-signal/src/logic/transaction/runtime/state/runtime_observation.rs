mod retained_charge;

use std::collections::BTreeSet;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::profile::DiagnosticsTier;

use super::runtime_state::SignalRuntime;

mod registry;

pub use registry::RuntimeObservationRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObserverId(u64);

impl ObserverId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ObservationHandleId(u64);

impl ObservationHandleId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationHandle {
    observer_id: ObserverId,
    handle_id: ObservationHandleId,
}

impl ObservationHandle {
    pub fn observer_id(self) -> ObserverId {
        self.observer_id
    }

    pub fn handle_id(self) -> ObservationHandleId {
        self.handle_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationTrigger {
    Touched,
    Recomputed,
    MeaningfulChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationDeliveryMode {
    PerCommittedTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationPolicy {
    trigger: ObservationTrigger,
    delivery_mode: ObservationDeliveryMode,
}

impl ObservationPolicy {
    pub fn new(trigger: ObservationTrigger, delivery_mode: ObservationDeliveryMode) -> Self {
        Self {
            trigger,
            delivery_mode,
        }
    }

    pub fn touched() -> Self {
        Self::new(
            ObservationTrigger::Touched,
            ObservationDeliveryMode::PerCommittedTransaction,
        )
    }

    pub fn recomputed() -> Self {
        Self::new(
            ObservationTrigger::Recomputed,
            ObservationDeliveryMode::PerCommittedTransaction,
        )
    }

    pub fn meaningful_change() -> Self {
        Self::new(
            ObservationTrigger::MeaningfulChange,
            ObservationDeliveryMode::PerCommittedTransaction,
        )
    }

    pub fn trigger(self) -> ObservationTrigger {
        self.trigger
    }

    pub fn delivery_mode(self) -> ObservationDeliveryMode {
        self.delivery_mode
    }
}

impl Default for ObservationPolicy {
    fn default() -> Self {
        Self::meaningful_change()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedNodeSet {
    nodes: BTreeSet<NodeId>,
}

impl ObservedNodeSet {
    pub fn from_nodes(nodes: impl IntoIterator<Item = NodeId>) -> Self {
        Self {
            nodes: nodes.into_iter().collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn contains(&self, node: NodeId) -> bool {
        self.nodes.contains(&node)
    }

    pub fn iter(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().copied()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MatchingObserverSet {
    observers: Vec<ObserverId>,
}

impl MatchingObserverSet {
    pub fn new(observers: Vec<ObserverId>) -> Self {
        Self { observers }
    }

    pub fn len(&self) -> usize {
        self.observers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observers.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = ObserverId> + '_ {
        self.observers.iter().copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationRegistrySummary {
    pub active_observer_count: usize,
    pub indexed_node_count: usize,
    pub ordered_observer_ids: Vec<ObserverId>,
}

pub struct ObservationNotice<'a> {
    observer_id: ObserverId,
    handle_id: ObservationHandleId,
    policy: ObservationPolicy,
    observed_nodes: &'a ObservedNodeSet,
    matched_nodes: &'a ObservedNodeSet,
    touched: bool,
    recomputed: bool,
    meaningful_change: bool,
    trigger_matched: bool,
}

impl ObservationNotice<'_> {
    pub fn observer_id(&self) -> ObserverId {
        self.observer_id
    }

    pub fn handle_id(&self) -> ObservationHandleId {
        self.handle_id
    }

    pub fn policy(&self) -> ObservationPolicy {
        self.policy
    }

    pub fn observed_nodes(&self) -> &ObservedNodeSet {
        self.observed_nodes
    }

    pub fn matched_nodes(&self) -> &ObservedNodeSet {
        self.matched_nodes
    }

    pub fn touched(&self) -> bool {
        self.touched
    }

    pub fn recomputed(&self) -> bool {
        self.recomputed
    }

    pub fn meaningful_change(&self) -> bool {
        self.meaningful_change
    }

    pub fn trigger_matched(&self) -> bool {
        self.trigger_matched
    }
}

pub trait ObservationListener<D, I, E, Ctx, T>: Send + Sync
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    fn on_observation(
        &self,
        ctx: ObservationReadContext<'_, D, I, E, Ctx, T>,
        notice: &ObservationNotice<'_>,
    );
}

pub struct ObservationReadContext<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    graph: &'a SignalGraph,
    _marker: PhantomData<(D, I, E, Ctx, T)>,
}

impl<'a, D, I, E, Ctx, T> ObservationReadContext<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(runtime: &'a SignalRuntime<D, I, E, Ctx, T>) -> Self {
        Self {
            graph: runtime.graph(),
            _marker: PhantomData,
        }
    }

    pub(in crate::logic::transaction::runtime) fn from_graph(graph: &'a SignalGraph) -> Self {
        Self {
            graph,
            _marker: PhantomData,
        }
    }

    pub fn graph(&self) -> crate::data::graph::GraphObserver<'a> {
        self.graph.observe()
    }

    pub fn diagnostics_profile(&self) -> DiagnosticsTier {
        self.graph.observe().diagnostics_profile()
    }

    pub fn runtime_policy(&self) -> crate::runtime_policy::SignalRuntimePolicy {
        self.graph.observe().runtime_policy()
    }

    pub fn current_branch(&self) -> crate::state::SignalBranchHandle {
        self.graph.current_branch()
    }
}
