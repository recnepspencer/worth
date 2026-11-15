//! Request-local input preparation belongs to the original installed producer.
use super::WorthQueryApplicationProducerBinding;
use crate::domain_computation::primary_graph::{
    DecisionReader, HandlerResult, OperationHandler, WorthQueryDecisionPlan,
};
use std::{any::Any, marker::PhantomData};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::WorthQueryProducerInputCaptureDefinition,
    application_schema::ApplicationSchema,
};
mod reader;
pub use reader::WorthQueryProducerInputReader;
mod contract;
pub(in crate::domain_computation::primary_graph) use contract::InputCaptureMeaning;
pub use contract::WorthQueryProducerInputCaptureContract;
mod owner;
pub(in crate::domain_computation::primary_graph::application_contribution) use owner::ErasedCapturedInputs;
pub(in crate::domain_computation::primary_graph::application_contribution) use owner::InputCaptureOwner;

type Operation<S, P> = <P as WorthQueryApplicationProducerBinding<S>>::Operation;
type Input<S, P> = <Operation<S, P> as ApplicationMutationBinding<S>>::Input;
type Decision<S, P> = <Operation<S, P> as ApplicationMutationBinding<S>>::Decision;
type Denial<S, P> = <Operation<S, P> as ApplicationMutationBinding<S>>::Denial;

/// A pure input prelude and the once-only decision consume the same owned data.
/// Capture may read and perform bounded numerical admission, but cannot publish,
/// invoke a producer, mutate supplier state or run the effectful computation.
pub trait WorthQueryProducerInputCaptureHandler<S, P, C>:
    OperationHandler<S, Operation<S, P>>
where
    S: ApplicationSchema,
    P: WorthQueryApplicationProducerBinding<S, OutputFamily = C::OutputFamily>,
    C: WorthQueryProducerInputCaptureDefinition<S>,
{
    type Captured: worth_execution::ChargedBytes + 'static;

    fn capture_inputs(
        &self,
        input: &Input<S, P>,
        reader: &mut WorthQueryProducerInputReader<'_, '_, '_, S, Operation<S, P>, C>,
    ) -> HandlerResult<WorthQueryCapturedProducerInputs<S, C, Self::Captured>, Denial<S, P>>;

    fn prepare_captured_decision(
        &self,
        input: &Input<S, P>,
        captured: WorthQueryCapturedProducerInputs<S, C, Self::Captured>,
        reader: &mut DecisionReader<'_, '_, '_, S, Operation<S, P>>,
    ) -> WorthQueryDecisionPlan<Decision<S, P>, Denial<S, P>>;
}

/// Produced only by a loan's `finish`; descriptive data grants no writes.
pub struct WorthQueryCapturedProducerInputs<S, C, Data> {
    pub(super) data: Data,
    pub(super) parents: Vec<CapturedParent>,
    pub(super) affinity: InputCaptureAffinity,
    pub(super) _storage: Option<super::super::request_execution::QueryMemoryReservation>,
    pub(super) _data_memory: Option<super::super::request_execution::QueryMemoryReservation>,
    marker: PhantomData<fn() -> (S, C)>,
}
impl<S, C, Data> WorthQueryCapturedProducerInputs<S, C, Data> {
    pub fn data(&self) -> &Data {
        &self.data
    }
    /// Consume the numerical payload after Query has drained every retained port.
    pub fn into_data(self) -> Data {
        self.data
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution) struct CapturedParent {
    pub(super) connection: std::any::TypeId,
    pub(super) source: worth_relational::facade::identity::EntityId,
    pub(super) demand: Box<dyn Any + Send>,
    pub(super) _memory: super::super::request_execution::QueryMemoryReservation,
}

/// Binding and candidate affinity come from the actual admitted projection.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_contribution) struct InputCaptureAffinity
{
    pub(super) admission:
        crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
    pub(super) candidate:
        worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification,
    pub(super) source: worth_relational::facade::identity::EntityId,
    pub(super) component: std::any::TypeId,
}
