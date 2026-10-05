use worth_execution::MapKernelContext;

use crate::data::error::SignalError;
use crate::data::handle::NodeId;
use crate::logic::checked_context::CheckedEvaluationContext;
use crate::logic::evaluation::IntoEvaluationOutput;
use crate::logic::prepared::{ExecutionReadView, PreparedEvaluation};

#[derive(Clone, Copy)]
pub(crate) struct CheckedKernelCapacity {
    pub(crate) scratch_bytes: u64,
    pub(crate) result_bytes: u64,
}

pub(crate) trait SignalPrecompute: Sync {
    fn prepare(
        &self,
        node: NodeId,
        view: &ExecutionReadView<'_>,
        work: Option<&mut MapKernelContext<'_, '_>>,
        capacity: Option<CheckedKernelCapacity>,
    ) -> Result<PreparedEvaluation, SignalError>;
    fn allows_bounded_inputs(&self) -> bool;
}

pub(crate) struct LegacyPrecompute<F>(F);

impl<F> LegacyPrecompute<F> {
    pub(crate) fn new(callback: F) -> Self {
        Self(callback)
    }
}

impl<F> SignalPrecompute for LegacyPrecompute<F>
where
    F: Fn(NodeId, &ExecutionReadView<'_>) -> Result<PreparedEvaluation, SignalError> + Sync,
{
    fn prepare(
        &self,
        node: NodeId,
        view: &ExecutionReadView<'_>,
        work: Option<&mut MapKernelContext<'_, '_>>,
        _capacity: Option<CheckedKernelCapacity>,
    ) -> Result<PreparedEvaluation, SignalError> {
        if work.is_some() {
            return Err(SignalError::invalid_input(
                "unbounded evaluator cannot enter checked dispatch",
            ));
        }
        (self.0)(node, view)
    }
    fn allows_bounded_inputs(&self) -> bool {
        false
    }
}

pub(crate) struct CheckedPrecompute<'a, F, Ctx> {
    domain: &'a Ctx,
    evaluator: &'a F,
}

impl<'a, F, Ctx> CheckedPrecompute<'a, F, Ctx> {
    pub(crate) fn new(domain: &'a Ctx, evaluator: &'a F) -> Self {
        Self { domain, evaluator }
    }
}

impl<F, Ctx, O> SignalPrecompute for CheckedPrecompute<'_, F, Ctx>
where
    Ctx: Sync,
    F: for<'graph, 'work, 'run, 'lease> Fn(
            &mut CheckedEvaluationContext<'graph, 'work, 'run, 'lease, Ctx>,
        ) -> Result<O, SignalError>
        + Sync,
    O: IntoEvaluationOutput,
{
    fn prepare(
        &self,
        node: NodeId,
        view: &ExecutionReadView<'_>,
        work: Option<&mut MapKernelContext<'_, '_>>,
        capacity: Option<CheckedKernelCapacity>,
    ) -> Result<PreparedEvaluation, SignalError> {
        let work = work.ok_or_else(|| {
            SignalError::invalid_input("checked evaluator needs its work context")
        })?;
        let capacity = capacity.ok_or_else(|| {
            SignalError::invalid_input("checked evaluator needs its admitted capacity")
        })?;
        let mut context = CheckedEvaluationContext::new(
            view.graph(),
            node,
            self.domain,
            work,
            capacity.scratch_bytes,
            capacity.result_bytes,
        )?;
        let output = (self.evaluator)(&mut context)?;
        context.into_prepared(output)
    }
    fn allows_bounded_inputs(&self) -> bool {
        true
    }
}
