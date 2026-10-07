use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::EvaluationRequestMode;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::planner::{
    build_evaluation_plan_with_policy_resolver, EvaluationPlan, ExecutionReport,
};

use super::super::super::state::SignalRuntime;
use super::super::shared::apply_strategy_maintenance;

use super::request::ExecutionIntent;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    pub fn target(
        &mut self,
        node: NodeId,
    ) -> super::request::RuntimeExecutionRequest<'_, D, I, E, Ctx, T> {
        super::request::RuntimeExecutionRequest::new(
            self,
            vec![node],
            EvaluationRequestMode::Default,
        )
    }

    pub fn targets(
        &mut self,
        nodes: impl IntoIterator<Item = NodeId>,
    ) -> super::request::RuntimeExecutionRequest<'_, D, I, E, Ctx, T> {
        super::request::RuntimeExecutionRequest::new(
            self,
            nodes.into_iter().collect(),
            EvaluationRequestMode::Default,
        )
    }

    pub fn build_evaluation_plan(
        &mut self,
        targets: &[NodeId],
        request_mode: EvaluationRequestMode,
    ) -> Result<EvaluationPlan, SignalError> {
        let mut resolver = crate::data::comparator::TierPolicyResolver::new(
            self.config.node_meta(),
            self.config.tier_policies(),
            self.config.fallback_comparator(),
        );
        build_evaluation_plan_with_policy_resolver(
            &mut self.graph,
            targets,
            request_mode,
            &mut resolver,
        )
    }

    pub fn evaluate_with_plan<F, O>(
        &mut self,
        node: NodeId,
        runtime_ctx: &Ctx,
        evaluator: &F,
        request_mode: EvaluationRequestMode,
    ) -> Result<ExecutionReport, SignalError>
    where
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        let strategy = self.derive_evaluation_strategy();
        let report = self.execute_evaluation(
            ExecutionIntent::Targets {
                targets: std::slice::from_ref(&node),
                request_mode,
            },
            runtime_ctx,
            evaluator,
            worth_execution::ExecutionRequest::serial(&self.graph.bounded_serial_request()),
        )?;
        apply_strategy_maintenance(&mut self.graph, strategy);
        Ok(report)
    }
}
