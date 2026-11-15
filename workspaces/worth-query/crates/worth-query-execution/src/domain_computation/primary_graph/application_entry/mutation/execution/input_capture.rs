//! Input data and original read custody live through the actual handler plan.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::QueryMemoryReservation, DecisionReader, OperationHandler,
    WorthQueryCompleteApplicationReadSet, WorthQueryDecisionPlan,
    WorthQueryProjectedApplicationMutation,
};
use std::any::Any;

type Reads<S, B> =
    WorthQueryCompleteApplicationReadSet<
        S,
        <B as ApplicationMutationBinding<S>>::Operation,
        <B as ApplicationMutationBinding<S>>::Input,
        <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<
            S,
        >>::Scope,
        WorthQueryProjectedApplicationMutation,
    >;
type Prepare<S, B> = fn(
    &dyn OperationHandler<S, B>,
    &<B as ApplicationMutationBinding<S>>::Input,
    Box<dyn Any>,
    &mut DecisionReader<'_, '_, '_, S, B>,
) -> WorthQueryDecisionPlan<
    <B as ApplicationMutationBinding<S>>::Decision,
    <B as ApplicationMutationBinding<S>>::Denial,
>;

pub(in crate::domain_computation::primary_graph) struct PreparedMutationInputs<S, B>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    value: Box<dyn Any>,
    retained: CapturedInputRetention<S, B>,
    prepare: Prepare<S, B>,
}

pub(super) struct CapturedInputRetention<S, B>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    reads: Reads<S, B>,
    memory: Option<QueryMemoryReservation>,
}

impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> PreparedMutationInputs<S, B> {
    pub(in crate::domain_computation::primary_graph) fn new(
        value: Box<dyn Any>,
        reads: Reads<S, B>,
        memory: Option<QueryMemoryReservation>,
        prepare: Prepare<S, B>,
    ) -> Self {
        Self {
            value,
            retained: CapturedInputRetention { reads, memory },
            prepare,
        }
    }

    pub(super) fn prepare(
        self,
        handler: &dyn OperationHandler<S, B>,
        input: &B::Input,
        reader: &mut DecisionReader<'_, '_, '_, S, B>,
    ) -> (
        WorthQueryDecisionPlan<B::Decision, B::Denial>,
        CapturedInputRetention<S, B>,
    ) {
        (
            (self.prepare)(handler, input, self.value, reader),
            self.retained,
        )
    }
}

impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> CapturedInputRetention<S, B> {
    pub(super) fn complete(
        self,
        reads: &mut Reads<S, B>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Option<QueryMemoryReservation>, WorthQueryApplicationAttemptDenial> {
        reads.append_captured_inputs(
            execution,
            self.reads,
            &runtime.runtime.authorization,
            allocation,
        )?;
        Ok(self.memory)
    }
}
