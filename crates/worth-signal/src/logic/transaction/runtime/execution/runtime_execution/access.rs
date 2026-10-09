use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::evaluation::IntoEvaluationOutput;

use super::super::super::state::SignalRuntime;
use super::super::request_order::requested_dependency_order;
use super::super::shared::apply_strategy_maintenance;

use super::request::ExecutionIntent;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn read<F, O>(
        &mut self,
        node: NodeId,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<AspectVersion, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let strategy = self.derive_evaluation_strategy();
        let version = self.read_serial(node, runtime_ctx, evaluator)?;
        apply_strategy_maintenance(&mut self.graph, strategy);
        Ok(version)
    }

    pub fn get<F, O>(
        &mut self,
        node: NodeId,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<AspectVersion, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        self.read(node, runtime_ctx, evaluator)
    }

    fn read_serial<F, O>(
        &mut self,
        node: NodeId,
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<AspectVersion, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let max_passes = self.graph.active_node_count().saturating_add(1);
        for _ in 0..max_passes {
            let mut scheduled = 0_u32;
            let mut executed = 0_u32;
            for target in requested_dependency_order(&self.graph, node)? {
                let report = self.execute_evaluation(
                    ExecutionIntent::Targets {
                        targets: std::slice::from_ref(&target),
                        request_mode: EvaluationRequestMode::Default,
                    },
                    runtime_ctx,
                    evaluator,
                    worth_execution::ExecutionRequest::serial(
                        &crate::data::host_execution::declared_serial_request(
                            self.graph.installed_runtime_policy().requested_policy(),
                        ),
                    ),
                )?;
                scheduled = scheduled.saturating_add(report.task_count);
                executed = executed.saturating_add(report.tasks_executed);
            }
            if scheduled == 0 || executed == 0 {
                return self.graph.node_aspect_version(node);
            }
        }
        Err(SignalError::internal(
            "runtime dependency settlement did not converge",
        ))
    }

    pub fn read_many<F, O>(
        &mut self,
        nodes: &[NodeId],
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<Vec<AspectVersion>, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let strategy = self.derive_evaluation_strategy();
        let versions = self.read_many_serial(nodes, runtime_ctx, evaluator)?;
        apply_strategy_maintenance(&mut self.graph, strategy);
        Ok(versions)
    }

    fn read_many_serial<F, O>(
        &mut self,
        nodes: &[NodeId],
        runtime_ctx: &Ctx,
        evaluator: &F,
    ) -> Result<Vec<AspectVersion>, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let pending = nodes
            .iter()
            .copied()
            .filter(|node| {
                !matches!(
                    self.graph.get_state(*node),
                    Ok(crate::data::node::NodeState::Clean)
                )
            })
            .collect::<Vec<_>>();
        if !pending.is_empty() {
            self.execute_evaluation(
                ExecutionIntent::Targets {
                    targets: &pending,
                    request_mode: EvaluationRequestMode::Default,
                },
                runtime_ctx,
                evaluator,
                worth_execution::ExecutionRequest::serial(
                    &crate::data::host_execution::declared_serial_request(
                        self.graph.installed_runtime_policy().requested_policy(),
                    ),
                ),
            )?;
        }
        nodes
            .iter()
            .copied()
            .map(|node| self.graph.node_aspect_version(node))
            .collect()
    }
}
