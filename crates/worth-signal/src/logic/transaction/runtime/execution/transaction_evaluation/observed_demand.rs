use std::collections::VecDeque;

use crate::data::bitset::DenseBitset;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::{EvaluationRequestMode, IntoEvaluationOutput};
use crate::logic::planner::StageExecutor;

use super::super::super::transaction::SignalTransaction;
use super::super::request_order::requested_dependency_order;
use super::super::shared::executor_for_strategy;

/// What one `evaluate_demand` / `evaluate_observed_demand` call did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObservedDemandSummary {
    /// Nodes visited walking subscribers forward from the staged candidates.
    pub reach_visits: u64,
    /// Demanded nodes (observed, or standing demand) selected as targets.
    pub targets: u32,
    /// Settlement passes run over the targets (0 when there were no targets).
    pub passes: u32,
    /// Tasks executed across all passes.
    pub tasks_executed: u32,
}

impl<'a, D, I, E, Ctx, T> SignalTransaction<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    /// Evaluates the observed nodes this transaction impacted, forcing nodes
    /// gated behind `EvaluationCondition::OnDemand`.
    ///
    /// Observation is demand. A watcher registered on a node has asked for
    /// that node's committed value, and `ObservationDeliveryMode::
    /// PerCommittedTransaction` only classifies what the committing
    /// transaction evaluated. `evaluate_dirty` runs in `Default` mode and
    /// leaves on-demand nodes `DeferredOnDemand`, so without this pass a
    /// watched on-demand computed is invalidated, never recomputed, and never
    /// delivered. Call it after `evaluate_dirty` and before commit.
    ///
    /// Impacted observed nodes are found by walking `subscribers_of` forward
    /// from every candidate this transaction staged (changed sources and the
    /// consumers admitted by output publication), so an unrelated watched
    /// node is never demanded. Each target is then settled like a read:
    /// every non-clean node in its dependency order is evaluated with
    /// `ForceOnDemand`, repeated until a pass executes nothing.
    ///
    /// Cost: O(downstream reach of the change) for the walk plus the
    /// evaluation of the non-clean upstream of each target; every part is
    /// counted in `TransactionTelemetry::observed_demand_*` and in the
    /// returned summary.
    pub fn evaluate_observed_demand<F, O>(
        &mut self,
        evaluator: &F,
    ) -> Result<ObservedDemandSummary, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.evaluate_demand(evaluator, &[])
    }

    /// `evaluate_observed_demand` extended with standing demand.
    ///
    /// `standing_demand` lists nodes whose committed value is always required
    /// even without a registered observer: a host that reads published
    /// outputs after every commit is an observer the registry does not know
    /// about. A standing node is demanded only when this transaction's staged
    /// candidates reach it, so an untouched output costs nothing beyond its
    /// bit in the membership set (O(len) to build). Dead ids are ignored.
    pub fn evaluate_demand<F, O>(
        &mut self,
        evaluator: &F,
        standing_demand: &[NodeId],
    ) -> Result<ObservedDemandSummary, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.evaluate_demand_with_executor(
            evaluator,
            standing_demand,
            executor_for_strategy(self.graph.derive_evaluation_strategy()),
        )
    }

    pub fn evaluate_demand_with_executor<F, O>(
        &mut self,
        evaluator: &F,
        standing_demand: &[NodeId],
        executor: StageExecutor,
    ) -> Result<ObservedDemandSummary, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let mut summary = ObservedDemandSummary::default();
        if self.observations.registration_count() == 0 && standing_demand.is_empty() {
            return Ok(summary);
        }
        let mut standing = DenseBitset::new();
        standing.ensure_len(self.graph.arena_capacity());
        for node in standing_demand {
            if self.graph.is_alive(*node) {
                standing.mark(node.index() as usize);
            }
        }
        let targets = self.impacted_demanded_nodes(&standing, &mut summary.reach_visits)?;
        summary.targets = targets.len() as u32;
        self.with_telemetry(|telemetry| {
            telemetry.transaction.observed_demand_reach_visits += summary.reach_visits;
            telemetry.transaction.observed_demand_targets += targets.len() as u64;
        });
        if targets.is_empty() {
            return Ok(summary);
        }

        // Same convergence bound as `read_with_executor`: each pass that
        // executes something settles at least one node, so a graph of N live
        // nodes never needs more than N passes.
        let max_passes = self.graph.active_node_count().saturating_add(1);
        for _ in 0..max_passes {
            summary.passes += 1;
            self.with_telemetry(|telemetry| telemetry.transaction.observed_demand_passes += 1);
            let mut scheduled = 0_u32;
            let mut executed = 0_u32;
            for target in &targets {
                for node in requested_dependency_order(self.graph, *target)? {
                    if matches!(self.graph.get_state(node)?, NodeState::Clean) {
                        continue;
                    }
                    let report = self.evaluate_with_plan_and_executor(
                        node,
                        evaluator,
                        EvaluationRequestMode::ForceOnDemand,
                        executor,
                    )?;
                    scheduled = scheduled.saturating_add(report.task_count);
                    executed = executed.saturating_add(report.tasks_executed);
                }
            }
            summary.tasks_executed = summary.tasks_executed.saturating_add(executed);
            if scheduled == 0 || executed == 0 {
                return Ok(summary);
            }
        }
        Err(SignalError::internal(
            "observed demand settlement did not converge",
        ))
    }

    /// Demanded nodes (observed, or marked in `standing`) reachable through
    /// `subscribers_of` from every candidate this transaction staged, in node
    /// order. `reach_visits` counts every node dequeued by the walk.
    fn impacted_demanded_nodes(
        &self,
        standing: &DenseBitset,
        reach_visits: &mut u64,
    ) -> Result<Vec<NodeId>, SignalError> {
        let mut seen = DenseBitset::new();
        seen.ensure_len(self.graph.arena_capacity());
        let mut queue = self
            .scratch
            .mark_dirty_staged
            .marked_indices()
            .into_iter()
            .filter_map(|index| self.graph.live_node_id_at(index))
            .collect::<VecDeque<_>>();
        let mut impacted = Vec::new();
        while let Some(node) = queue.pop_front() {
            if !seen.mark(node.index() as usize) {
                continue;
            }
            *reach_visits += 1;
            if standing.contains(node.index() as usize)
                || self.observations.has_matching_observers_for_node(node)
            {
                impacted.push(node);
            }
            for subscriber in self.graph.subscribers_of(node)? {
                if self.graph.is_alive(*subscriber) && !seen.contains(subscriber.index() as usize) {
                    queue.push_back(*subscriber);
                }
            }
        }
        impacted.sort_unstable();
        Ok(impacted)
    }
}
