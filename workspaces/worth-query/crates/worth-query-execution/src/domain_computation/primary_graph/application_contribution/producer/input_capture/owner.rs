//! Typed registry thunks borrow the original handler; no configured owner is copied.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    QueryRequestExecution, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
};
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryApplicationProjection, WorthQueryInvariantEntityIdentity,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::equality_type_id::EqualityTypeId;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;

type Scope<S, P> = <<Operation<S, P> as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
type Op<S, P> = <Operation<S, P> as ApplicationMutationBinding<S>>::Operation;
type Capture<S, P> =
    for<'call, 'reader, 'runtime> fn(
        &'call WorthQueryPrimaryGraphApplicationRuntime<S>,
        &'call Input<S, P>,
        &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
            'reader,
            'runtime,
            S,
            Op<S, P>,
        >,
        &'call WorthQueryInvariantEntityIdentity<S, Scope<S, P>>,
        &'call QueryRequestExecution<'call>,
        &'call [std::any::TypeId],
        InputCaptureAffinity,
    ) -> HandlerResult<ErasedCapturedInputs, Denial<S, P>>;
type Prepare<S, P> = fn(
    &dyn OperationHandler<S, Operation<S, P>>,
    &Input<S, P>,
    Box<dyn Any>,
    &mut DecisionReader<'_, '_, '_, S, Operation<S, P>>,
) -> WorthQueryDecisionPlan<Decision<S, P>, Denial<S, P>>;

pub(in crate::domain_computation::primary_graph::application_contribution) struct InputCaptureOwner<
    S,
    P,
> where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S>,
{
    pub(in crate::domain_computation::primary_graph::application_contribution) definition:
        EqualityTypeId,
    pub(in crate::domain_computation::primary_graph::application_contribution) capture:
        Capture<S, P>,
    pub(in crate::domain_computation::primary_graph::application_contribution) prepare:
        Prepare<S, P>,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) admit:
        super::super::execution::CaptureAdmission<S, P>,
}
impl<S, P> Copy for InputCaptureOwner<S, P>
where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S>,
{
}
impl<S, P> Clone for InputCaptureOwner<S, P>
where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S>,
{
    fn clone(&self) -> Self {
        *self
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution) struct ErasedCapturedInputs
{
    pub(in crate::domain_computation::primary_graph::application_contribution) affinity:
        InputCaptureAffinity,
    value: Box<dyn Any>,
    parents: fn(&dyn Any) -> &[CapturedParent],
    memory: Option<super::super::super::request_execution::QueryMemoryReservation>,
    context_use: crate::domain_computation::primary_graph::handler::DecisionContextUse,
}
impl ErasedCapturedInputs {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn parents(
        &self,
    ) -> &[CapturedParent] {
        (self.parents)(self.value.as_ref())
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn into_parts(
        self,
    ) -> (
        Box<dyn Any>,
        Option<super::super::super::request_execution::QueryMemoryReservation>,
        crate::domain_computation::primary_graph::handler::DecisionContextUse,
    ) {
        (self.value, self.memory, self.context_use)
    }
}

impl<S, P> InputCaptureOwner<S, P>
where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S>,
{
    pub(in crate::domain_computation::primary_graph::application_contribution) fn new<C, H>() -> Self
    where
        C: WorthQueryProducerInputCaptureDefinition<S, OutputFamily = P::OutputFamily>,
        H: WorthQueryProducerInputCaptureHandler<S, P, C>,
        super::super::ProducerSourceValue<S, P>:
            WorthQueryApplicationProjection<S, super::super::ProducerSourceQuery<S, P>>,
    {
        Self {
            definition: EqualityTypeId::of::<C>(),
            capture: capture::<S, P, C, H>,
            prepare: prepare::<S, P, C, H>,
            admit: super::super::execution::capture_producer_inputs::<S, P>,
        }
    }
}

fn foreign() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::ForeignDemand,
        "producer input owner",
    ))
}

fn capture<'call, 'reader, 'runtime, S, P, C, H>(
    runtime: &'call WorthQueryPrimaryGraphApplicationRuntime<S>,
    input: &'call Input<S, P>,
    reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        S,
        Op<S, P>,
    >,
    scope: &'call WorthQueryInvariantEntityIdentity<S, Scope<S, P>>,
    execution: &'call QueryRequestExecution<'call>,
    ports: &'call [std::any::TypeId],
    affinity: InputCaptureAffinity,
) -> HandlerResult<ErasedCapturedInputs, Denial<S, P>>
where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S>,
    C: WorthQueryProducerInputCaptureDefinition<S, OutputFamily = P::OutputFamily>,
    H: WorthQueryProducerInputCaptureHandler<S, P, C>,
{
    let Some(handler) = runtime
        .mutation_handlers
        .original_handler::<Operation<S, P>, H>()
    else {
        return HandlerResult::ExecutionDenied(foreign());
    };
    let mut loan = WorthQueryProducerInputReader::<S, Operation<S, P>, C>::new(
        reader, scope, execution, ports, affinity,
    );
    let result = handler.capture_inputs(input, &mut loan);
    let context_use = loan.context_use();
    match loan.complete_call(result) {
        HandlerResult::Completed(mut captured) => {
            let memory = captured._data_memory.take();
            HandlerResult::Completed(ErasedCapturedInputs {
                affinity: captured.affinity,
                value: Box::new(captured),
                parents: parents::<S, C, H::Captured>,
                memory,
                context_use,
            })
        }
        HandlerResult::DomainDenied(cause) => HandlerResult::DomainDenied(cause),
        HandlerResult::ExecutionDenied(cause) => HandlerResult::ExecutionDenied(cause),
        HandlerResult::Cancelled => HandlerResult::Cancelled,
        HandlerResult::DeadlineExceeded => HandlerResult::DeadlineExceeded,
    }
}

fn parents<S: 'static, C: 'static, Data: 'static>(value: &dyn Any) -> &[CapturedParent] {
    &value
        .downcast_ref::<WorthQueryCapturedProducerInputs<S, C, Data>>()
        .expect("the selected capture thunk retains its concrete sealed payload")
        .parents
}

fn prepare<S, P, C, H>(
    handler: &dyn OperationHandler<S, Operation<S, P>>,
    input: &Input<S, P>,
    captured: Box<dyn Any>,
    reader: &mut DecisionReader<'_, '_, '_, S, Operation<S, P>>,
) -> WorthQueryDecisionPlan<Decision<S, P>, Denial<S, P>>
where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S>,
    C: WorthQueryProducerInputCaptureDefinition<S, OutputFamily = P::OutputFamily>,
    H: WorthQueryProducerInputCaptureHandler<S, P, C>,
{
    let original: &dyn Any = handler;
    match (
        original.downcast_ref::<H>(),
        captured.downcast::<WorthQueryCapturedProducerInputs<S, C, H::Captured>>(),
    ) {
        (Some(handler), Ok(inputs)) => handler.prepare_captured_decision(input, *inputs, reader),
        _ => WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(foreign())),
    }
}
