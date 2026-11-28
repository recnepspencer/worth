use super::*;
use crate::data::temporal::{TemporalPreviousValueReference, TemporalWakeId};
use crate::logic::transaction::TransactionResult;

impl<T: Copy + Ord, F> DefinedKeyedComputation<'_, T, F> {
    /// Evaluates the installed key and publishes ordinary transaction observations.
    pub fn commit_evaluation<D, I, E, Ctx, O>(
        &self,
        runtime: &mut SignalRuntime<D, I, E, Ctx, T>,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        context: &mut Ctx,
        memo_key: Option<&StructuralMemoKey>,
    ) -> Result<TransactionResult, SignalError>
    where
        D: Copy + Ord + std::fmt::Debug + 'static,
        I: Copy + Ord,
        Ctx: Sync,
        F: for<'ctx> Fn(&mut EvaluationContext<'ctx, Ctx>) -> Result<O, SignalError> + Sync,
        O: IntoEvaluationOutput,
    {
        if runtime.owner_services.is_sealed() {
            return Err(SignalError::invalid_input(
                "Signal evaluation requires its sealed owner service",
            ));
        }
        runtime.transaction(execution, context, |transaction| match memo_key {
            Some(key) => self.evaluate_memoized(transaction, key.clone()),
            None => self.evaluate(transaction),
        })
    }
}

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    Ctx: Sync,
    T: Copy + Ord,
{
    /// Captures the previous-value reference within the transaction that uses
    /// it. Signal checks wake liveness and records the reference as evidence.
    pub fn commit_evaluation_with_temporal_previous_value<O>(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        context: &mut Ctx,
        node: NodeId,
        wake: TemporalWakeId,
        evaluator: impl for<'ctx> Fn(
                &mut EvaluationContext<'ctx, Ctx>,
                &TemporalPreviousValueReference,
            ) -> Result<O, SignalError>
            + Sync,
    ) -> Result<TransactionResult, SignalError>
    where
        O: IntoEvaluationOutput,
    {
        if self.owner_services.is_sealed() {
            return Err(SignalError::invalid_input(
                "Signal evaluation requires its sealed owner service",
            ));
        }
        self.transaction(execution, context, |transaction| {
            let access = transaction.grant_temporal_previous_value_access(wake)?;
            let previous = transaction.previous_temporal_value(&access, node)?;
            transaction.evaluate_with_plan(
                node,
                &|context| evaluator(context, &previous),
                EvaluationRequestMode::ForceOnDemand,
            )?;
            Ok(())
        })
    }
}
