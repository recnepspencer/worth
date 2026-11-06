use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::ExecutionReport;
use worth_execution::ExecutionResourceLease;

use super::super::super::state::SignalRuntime;

pub(super) enum ExecutionIntent<'a> {
    Targets {
        targets: &'a [NodeId],
        request_mode: EvaluationRequestMode,
    },
    Dirty,
}

pub struct RuntimeExecutionRequest<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>,
    targets: Vec<NodeId>,
    request_mode: EvaluationRequestMode,
}

impl<'a, D, I, E, Ctx, T> RuntimeExecutionRequest<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub(super) fn new(
        runtime: &'a mut SignalRuntime<D, I, E, Ctx, T>,
        targets: Vec<NodeId>,
        request_mode: EvaluationRequestMode,
    ) -> Self {
        Self {
            runtime,
            targets,
            request_mode,
        }
    }

    /// Forces `run` to re-execute the requested targets even when they are
    /// current. Reads ignore it: a read never recomputes a current value.
    pub fn on_demand(mut self) -> Self {
        self.request_mode = EvaluationRequestMode::ForceOnDemand;
        self
    }

    pub fn with_mode(mut self, request_mode: EvaluationRequestMode) -> Self {
        self.request_mode = request_mode;
        self
    }

    pub fn run<F, O>(self, runtime_ctx: &Ctx, evaluator: &F) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.runtime.execute_evaluation(
            ExecutionIntent::Targets {
                targets: &self.targets,
                request_mode: self.request_mode,
            },
            runtime_ctx,
            evaluator,
            worth_execution::ExecutionRequest::serial(
                &crate::data::host_execution::declared_serial_request(
                    self.runtime
                        .graph
                        .installed_runtime_policy()
                        .requested_policy(),
                ),
            ),
        )
    }

    pub fn run_checked<F, O>(
        self,
        runtime_ctx: &Ctx,
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
        self.runtime.evaluate_checked(
            &self.targets,
            self.request_mode,
            runtime_ctx,
            evaluator,
            lease,
        )
    }

    /// Reads one target. Like `read_many`, it settles only what is not
    /// current and ignores the request mode.
    pub fn read<F, O>(
        mut self,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<AspectVersion, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let [node] = self.targets.as_slice() else {
            return Err(SignalError::invalid_input(
                "guided read requires exactly one target; use read_many for multiple targets",
            ));
        };
        let node = *node;
        self.settle(runtime_ctx, evaluator)?;
        self.runtime.graph.node_aspect_version(node)
    }

    /// Reads every target after settling what is not current.
    pub fn read_many<F, O>(
        mut self,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<Vec<AspectVersion>, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.settle(runtime_ctx, evaluator)?;
        self.targets
            .iter()
            .map(|node| self.runtime.graph.node_aspect_version(*node))
            .collect()
    }

    /// A clean node is only locally clean: it is current when nothing above
    /// it is unsettled, which the planner decides from the whole upstream
    /// path. Reads therefore always plan in `Default` mode, which keeps
    /// current targets out of the plan; forcing applies only to `run`.
    fn settle<F, O>(&mut self, runtime_ctx: &Ctx, evaluator: &F) -> Result<(), SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.runtime.execute_evaluation(
            ExecutionIntent::Targets {
                targets: &self.targets,
                request_mode: EvaluationRequestMode::Default,
            },
            runtime_ctx,
            evaluator,
            worth_execution::ExecutionRequest::serial(
                &crate::data::host_execution::declared_serial_request(
                    self.runtime
                        .graph
                        .installed_runtime_policy()
                        .requested_policy(),
                ),
            ),
        )?;
        Ok(())
    }
}
