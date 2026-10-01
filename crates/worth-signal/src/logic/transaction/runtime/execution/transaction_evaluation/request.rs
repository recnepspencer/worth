use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::{EvaluationRequestMode, IntoEvaluationOutput};
use crate::logic::planner::ExecutionReport;
use worth_execution::ExecutionResourceLease;

use super::super::super::transaction::SignalTransaction;

pub(super) enum TransactionExecutionIntent<'a> {
    Targets {
        targets: &'a [NodeId],
        request_mode: EvaluationRequestMode,
        stage_task_candidates: bool,
    },
    Dirty,
}

pub struct TransactionExecutionRequest<'tx, 'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    tx: &'tx mut SignalTransaction<'a, D, I, E, Ctx, T>,
    targets: Vec<NodeId>,
    request_mode: EvaluationRequestMode,
}

impl<'tx, 'a, D, I, E, Ctx, T> TransactionExecutionRequest<'tx, 'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub(super) fn new(
        tx: &'tx mut SignalTransaction<'a, D, I, E, Ctx, T>,
        targets: Vec<NodeId>,
        request_mode: EvaluationRequestMode,
    ) -> Self {
        Self {
            tx,
            targets,
            request_mode,
        }
    }

    pub fn on_demand(mut self) -> Self {
        self.request_mode = EvaluationRequestMode::ForceOnDemand;
        self
    }

    pub fn with_mode(mut self, request_mode: EvaluationRequestMode) -> Self {
        self.request_mode = request_mode;
        self
    }

    pub fn run<F, O>(self, evaluator: &F) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.tx.execute_evaluation(
            TransactionExecutionIntent::Targets {
                targets: &self.targets,
                request_mode: self.request_mode,
                stage_task_candidates: false,
            },
            evaluator,
            None,
        )
    }

    pub fn run_checked<F, O>(
        self,
        evaluator: &F,
        lease: &ExecutionResourceLease<'_>,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'graph, 'work, 'run, 'lease> Fn(
                &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
            ) -> Result<O, SignalError>
            + Sync,
        O: IntoEvaluationOutput,
    {
        self.tx
            .evaluate_checked(&self.targets, self.request_mode, evaluator, lease)
    }

    pub fn read<F, O>(self, evaluator: &F) -> Result<AspectVersion, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let [node] = self.targets.as_slice() else {
            return Err(SignalError::invalid_input(
                "guided read requires exactly one target; use read_many for multiple targets",
            ));
        };
        if !matches!(
            self.tx.graph.get_state(*node)?,
            crate::data::node::NodeState::Clean
        ) {
            self.tx.execute_evaluation(
                TransactionExecutionIntent::Targets {
                    targets: &self.targets,
                    request_mode: self.request_mode,
                    stage_task_candidates: false,
                },
                evaluator,
                None,
            )?;
        }
        self.tx.graph.node_aspect_version(*node)
    }

    pub fn read_many<F, O>(self, evaluator: &F) -> Result<Vec<AspectVersion>, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let pending = self
            .targets
            .iter()
            .copied()
            .filter(|node| {
                !matches!(
                    self.tx.graph.get_state(*node),
                    Ok(crate::data::node::NodeState::Clean)
                )
            })
            .collect::<Vec<_>>();
        if !pending.is_empty() {
            self.tx.execute_evaluation(
                TransactionExecutionIntent::Targets {
                    targets: &pending,
                    request_mode: self.request_mode,
                    stage_task_candidates: false,
                },
                evaluator,
                None,
            )?;
        }
        self.targets
            .into_iter()
            .map(|node| self.tx.graph.node_aspect_version(node))
            .collect()
    }
}
