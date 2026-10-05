use super::super::types::{EvaluationPlan, ExecutionReport};
use super::execute_prepared_plan_with_policy;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::logic::context::EvaluationContext;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::prepared::PreparedEvaluation;

pub(super) fn prepare_with_context<Ctx, F, O>(
    graph: &SignalGraph,
    domain_ctx: &Ctx,
    node: NodeId,
    evaluator: &F,
) -> Result<PreparedEvaluation, SignalError>
where
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    let mut eval_ctx = EvaluationContext::new(graph, node, domain_ctx);
    let output = evaluator(&mut eval_ctx)?;
    Ok(eval_ctx.into_prepared(output))
}

pub fn execute_prepared_plan<Ctx, F, O>(
    graph: &mut SignalGraph,
    plan: &EvaluationPlan,
    domain_ctx: &Ctx,
    evaluator: &F,
) -> Result<ExecutionReport, SignalError>
where
    Ctx: Sync,
    F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
    O: IntoEvaluationOutput,
{
    let mut comparator = crate::data::comparator::DefaultComparatorResolver;
    let mut resolver = crate::data::comparator::DefaultComparatorPolicyResolver {
        fallback: crate::data::comparator::VersionComparatorPolicy::Exact,
        custom: &mut comparator,
    };
    execute_prepared_plan_with_policy(graph, plan, domain_ctx, evaluator, &mut resolver, None)
}
