//! One cumulative node edit for an installed dependency replacement.
use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::runtime::graph::{
    map_node_edit, map_node_edit_accounting, map_node_edit_retention, NodeArena,
    PreparedRetainedNodeEdit, RetainedNodeEditOutcome, RetainedNodeEditPreparation,
};
use crate::data::graph::signal_graph::DependencyTopologyDelta;
use crate::data::graph::storage::NodeEvaluationMutation;
use crate::data::graph::{DependencySetId, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, PendingDependencyRevalidation,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
};
use crate::logic::evaluation::EvaluationWork;

struct TopologyNodeUpdate {
    dependencies: DependencySetId,
    pending: PendingDependencyRevalidation,
}

enum PreparedTopologyNodeUpdate {
    Direct(TopologyNodeUpdate),
    Retained(PreparedRetainedNodeEdit<()>),
}

impl SignalGraph {
    pub(super) fn set_dependency_edges_sorted_with_delta(
        &mut self,
        node: NodeId,
        edges: &[DependencyEdge],
        delta: DependencyTopologyDelta,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        self.validate_handle(node)?;
        let previous = self.node_pending_revalidation(node)?;
        let previous_len = previous.map_or(0, |pending| pending.unresolved_producers().len());
        let sort_steps = (usize::BITS - edges.len().leading_zeros()) as usize + 1;
        work.reserve(
            previous_len
                .checked_add(edges.len())
                .and_then(|n| n.checked_mul(8)),
        )?;
        work.reserve(
            edges
                .len()
                .checked_mul(sort_steps)
                .and_then(|n| n.checked_mul(2)),
        )?;
        let projection_charge = Charge::capacity::<NodeId>(previous_len)
            .and_then(|charge| {
                charge.checked_add(Charge::capacity::<NodeId>(edges.len())?.checked_mul(2)?)
            })
            .map_err(map_node_edit_accounting)?;
        let _projection_custody = self
            .arena
            .retained_node_ledger
            .as_ref()
            .map(|ledger| {
                ledger
                    .reserve(0, projection_charge)
                    .map_err(map_node_edit_retention)
            })
            .transpose()?;
        let previous = previous
            .map(|pending| pending.unresolved_producers().to_vec())
            .unwrap_or_default();
        let mut producers = Vec::with_capacity(edges.len());
        for edge in edges {
            if !matches!(self.get_state(edge.source()), Ok(NodeState::Clean)) {
                producers.push(edge.source());
            }
        }
        let revision = DependencyRevision(
            self.node_dependency_revision(node)?
                .0
                .checked_add(1)
                .ok_or_else(|| SignalError::internal("dependency revision overflow"))?,
        );
        let pending = PendingDependencyRevalidation::structural(revision, producers);
        let current = self.pending_cause_set_id(node)?;
        self.cause_sets.admit_release_work(current, work)?;
        let reverse = self.prepare_reverse_subscription_replacement(node, edges, work)?;
        let maximum = self
            .installed_runtime_policy()
            .conditional_evaluation_budget();
        let insertion = self.topology.dependency_edges.prepare_insertion(edges);
        let update = TopologyNodeUpdate {
            dependencies: insertion.id(),
            pending,
        };
        let prepared = prepare_node_update(&self.arena, node, update, maximum, work)?;
        // Every fallible node-storage admission precedes cause release and the
        // selected segment's publication. The segment borrow fixes its handle.
        if current != crate::data::graph::storage::invalidation_causes::PendingCauseSetId::EMPTY {
            self.cause_sets.release(current)?;
        }
        insertion.publish();
        prepared.publish(&mut self.arena, node);
        reverse.publish(self);
        // Borrowed node payload cannot survive mutation of the waiter index.
        let current = self
            .node_pending_revalidation(node)?
            .expect("installed structural projection")
            .unresolved_producers()
            .to_vec();
        self.replace_pending_revalidation_waiters(node, &previous, &current);
        self.record_branch_mutation_dependencies(node, delta);
        self.record_graph_storage_pressure();
        Ok(())
    }
}

fn prepare_node_update(
    arena: &NodeArena,
    node: NodeId,
    update: TopologyNodeUpdate,
    maximum: crate::runtime_policy::SignalConditionalEvaluationBudget,
    work: &mut EvaluationWork<'_>,
) -> Result<PreparedTopologyNodeUpdate, SignalError> {
    let Some(ledger) = &arena.retained_node_ledger else {
        return Ok(PreparedTopologyNodeUpdate::Direct(update));
    };
    let bytes = usize::try_from(maximum.maximum_retained_bytes)
        .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
    let ceiling = Charge::capacity::<u8>(bytes).map_err(map_node_edit_accounting)?;
    let prepare = |work: &mut Work| {
        arena
            .prepare_retained_node_edits(
                ledger,
                &[node.index() as usize],
                ceiling,
                work,
                |payloads| {
                    let payload = &mut payloads[0];
                    update.apply(&mut NodeEvaluationMutation::draft(
                        &mut payload.hot,
                        &mut payload.warm,
                        &mut payload.cold,
                    ));
                },
            )
            .map_err(map_node_edit)
    };
    let prepared = match work {
        EvaluationWork::Conditional(work) => prepare(work)?,
        EvaluationWork::Ordinary => prepare(&mut Work::new(maximum.maximum_attempt_visits))?,
    };
    match prepared {
        RetainedNodeEditPreparation::Ready(prepared) => {
            Ok(PreparedTopologyNodeUpdate::Retained(prepared))
        }
        RetainedNodeEditPreparation::Rejected { denial, .. } => Err(map_node_edit(denial)),
    }
}

impl TopologyNodeUpdate {
    fn apply(self, target: &mut NodeEvaluationMutation<'_>) {
        target.replace_dependency_topology(self.dependencies, self.pending);
    }
}
impl PreparedTopologyNodeUpdate {
    fn publish(self, arena: &mut NodeArena, node: NodeId) {
        match self {
            Self::Direct(update) => update.apply(&mut NodeEvaluationMutation::installed(
                arena,
                node.index() as usize,
            )),
            Self::Retained(prepared) => match prepared.install(arena) {
                RetainedNodeEditOutcome::Installed { .. } => (),
                RetainedNodeEditOutcome::Rejected { .. } => {
                    unreachable!("exclusive topology node preparation")
                }
            },
        }
    }
}
