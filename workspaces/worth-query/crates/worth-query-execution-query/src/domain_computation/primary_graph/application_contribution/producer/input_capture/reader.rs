//! A pure loan exposes the existing tracked readonly vocabulary and fixed ports.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    QueryRequestExecution, WorthQueryManagedComputationInterruption,
    WorthQueryManagedComputationResourceDenial,
};
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, HandlerInterruption, WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationScopeBinding,
    application_program::{ApplicationConnectionShape, WorthQueryProducerOutputFamily},
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::{ApplicationEntityMarkerIdentity, OperationReads},
};
mod content;
mod control;
mod reads;
mod structure;
type Scope<S, B> = <<B as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;
type Parent<S, E> = <<E as ApplicationConnectionShape<S>>::Binding as WorthQueryApplicationDependentOutputConnection<S>>::RootDemand;

pub struct WorthQueryProducerInputReader<'call, 'reader, 'runtime, S, B, C>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
{
    reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        S,
        B::Operation,
    >,
    scope: &'call WorthQueryInvariantEntityIdentity<S, Scope<S, B>>,
    execution: &'call QueryRequestExecution<'call>,
    ports: &'call [std::any::TypeId],
    affinity: InputCaptureAffinity,
    parents: Option<Vec<CapturedParent>>,
    storage: Option<super::super::super::request_execution::QueryMemoryReservation>,
    stopped: std::cell::RefCell<Option<InputCaptureStop>>,
    context_use:
        std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>,
    marker: PhantomData<fn() -> C>,
}

#[derive(Clone, Debug)]
pub enum InputCaptureStop {
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(HandlerInterruption),
    ForeignPort,
    Finished,
    ForeignCapture,
    Work(crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial),
    Read(std::sync::Arc<HandlerExecutionDenial>),
}
impl std::fmt::Display for InputCaptureStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resource(cause) => write!(f, "producer input preparation: Resource({cause:?})"),
            Self::Work(cause) => write!(f, "producer input preparation: Work({cause:?})"),
            _ => write!(f, "producer input preparation: {self:?}"),
        }
    }
}
impl std::error::Error for InputCaptureStop {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}

impl<'call, 'reader, 'runtime, S, B, C>
    WorthQueryProducerInputReader<'call, 'reader, 'runtime, S, B, C>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    C: WorthQueryProducerInputCaptureDefinition<S>,
{
    pub fn new(
        reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
            'reader,
            'runtime,
            S,
            B::Operation,
        >,
        scope: &'call WorthQueryInvariantEntityIdentity<S, Scope<S, B>>,
        execution: &'call QueryRequestExecution<'call>,
        ports: &'call [std::any::TypeId],
        affinity: InputCaptureAffinity,
    ) -> Self {
        Self {
            reader,
            scope,
            execution,
            ports,
            affinity,
            parents: Some(Vec::new()),
            storage: None,
            stopped: std::cell::RefCell::new(None),
            context_use: std::cell::Cell::new(Default::default()),
            marker: PhantomData,
        }
    }

    pub fn scope(&self) -> &WorthQueryInvariantEntityIdentity<S, Scope<S, B>> {
        self.context_use.set(self.context_use.get().scope());
        self.scope
    }

    pub fn context_use(
        &self,
    ) -> crate::domain_computation::primary_graph::handler::DecisionContextUse {
        self.context_use.get()
    }

    /// Retain a genuine read-issued source and its one declared parent demand.
    /// Query validates the resolved demand's full route and drains every entry.
    #[expect(
        clippy::disallowed_methods,
        reason = "TypeId matches the exact declared port; it never orders parents"
    )]
    pub fn parent<E, Entity>(
        &mut self,
        source: &WorthQueryInvariantEntityIdentity<S, Entity>,
        demand: Parent<S, E>,
    ) -> Result<(), HandlerExecutionDenial>
    where
        E: ApplicationConnectionShape<S>,
        E::Binding: WorthQueryApplicationDependentOutputConnection<S>,
        Parent<S, E>: worth_execution::ChargedBytes + Send,
        Entity: ApplicationEntityMarkerIdentity<S> + OperationReads<B::Operation>,
        <<Parent<S, E> as super::super::WorthQueryApplicationOutputDemand<S>>::OutputFamily as WorthQueryProducerOutputFamily<S>>::Source:
            ApplicationQueryBinding<S, ScopeBinding: ApplicationQueryScopeBinding<S, Scope = Entity>>,
{
        self.charge(self.ports.len() as u64 + 1)?;
        if !self.ports.contains(&std::any::TypeId::of::<E>()) {
            return Err(self.stop(InputCaptureStop::ForeignPort));
        }
        let source = self.reader.captured_input_source(source);
        let source = self.checked(source)?;
        let bytes = (std::mem::size_of_val(&demand) as u64)
            .checked_add(worth_execution::ChargedBytes::additional_charged_bytes(
                &demand,
            ))
            .ok_or_else(|| {
                self.stop(InputCaptureStop::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
        let memory = self
            .execution
            .reserve(bytes)
            .map_err(|cause| self.stop(InputCaptureStop::Resource(cause)))?;
        if self.parents.is_none() {
            return Err(self.stop(InputCaptureStop::Finished));
        }
        let parents = self.parents.as_ref().expect("checked unfinished");
        if parents.len() == parents.capacity() {
            let capacity = parents.capacity().max(1).checked_mul(2).ok_or_else(|| {
                self.stop(InputCaptureStop::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
            let bytes = capacity
                .checked_mul(std::mem::size_of::<CapturedParent>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(|| {
                    self.stop(InputCaptureStop::Resource(
                        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                    ))
                })?;
            self.charge(bytes)?;
            let storage = self
                .execution
                .reserve(bytes)
                .map_err(|cause| self.stop(InputCaptureStop::Resource(cause)))?;
            let mut replacement = Vec::new();
            replacement.try_reserve_exact(capacity).map_err(|_| {
                self.stop(InputCaptureStop::Resource(
                    WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                ))
            })?;
            if replacement.capacity() != capacity {
                return Err(self.stop(InputCaptureStop::Resource(
                    WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                )));
            }
            replacement.append(self.parents.as_mut().expect("unfinished"));
            self.parents = Some(replacement);
            self.storage = Some(storage);
        }
        self.parents
            .as_mut()
            .expect("unfinished")
            .push(CapturedParent {
                connection: std::any::TypeId::of::<E>(),
                source,
                demand: Box::new(demand),
                _memory: memory,
            });
        Ok(())
    }

    /// Seal this loan's complete ports together with the owned numerical payload.
    pub fn finish<Data: worth_execution::ChargedBytes + 'static>(
        &mut self,
        data: Data,
    ) -> Result<WorthQueryCapturedProducerInputs<S, C, Data>, HandlerExecutionDenial> {
        self.charge(1)?;
        if let Some(stop) = self.stopped.borrow().as_ref() {
            return Err(HandlerExecutionDenial::new(stop.clone()));
        }
        let bytes = (std::mem::size_of::<WorthQueryCapturedProducerInputs<S, C, Data>>() as u64)
            .checked_add(data.additional_charged_bytes())
            .ok_or_else(|| {
                self.stop(InputCaptureStop::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
        let memory = self
            .execution
            .reserve(bytes)
            .map_err(|cause| self.stop(InputCaptureStop::Resource(cause)))?;
        let parents = self
            .parents
            .take()
            .ok_or_else(|| self.stop(InputCaptureStop::Finished))?;
        Ok(WorthQueryCapturedProducerInputs {
            data,
            parents,
            affinity: self.affinity,
            _storage: self.storage.take(),
            _data_memory: Some(memory),
            marker: PhantomData,
        })
    }

    fn stop(&self, cause: InputCaptureStop) -> HandlerExecutionDenial {
        let first = self.stopped.borrow_mut().get_or_insert(cause).clone();
        HandlerExecutionDenial::new(first)
    }

    /// The invocation owner checks the loan after the supplier returns, including
    /// caught failures and a container retained from a different invocation.
    pub fn complete_call<Data, Denial>(
        self,
        result: HandlerResult<WorthQueryCapturedProducerInputs<S, C, Data>, Denial>,
    ) -> HandlerResult<WorthQueryCapturedProducerInputs<S, C, Data>, Denial> {
        let _ = self.checkpoint();
        if let Some(stop) = self.stopped.into_inner() {
            return match stop {
                InputCaptureStop::Interrupted(HandlerInterruption::Cancelled) => {
                    HandlerResult::Cancelled
                }
                InputCaptureStop::Interrupted(HandlerInterruption::DeadlineExceeded) => {
                    HandlerResult::DeadlineExceeded
                }
                cause => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            };
        }
        if let HandlerResult::Completed(captured) = &result
            && (self.parents.is_some() || captured.affinity != self.affinity)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                InputCaptureStop::ForeignCapture,
            ));
        }
        result
    }

    fn checked<T, E: std::error::Error + Send + Sync + 'static>(
        &self,
        result: Result<T, E>,
    ) -> Result<T, HandlerExecutionDenial> {
        result.map_err(|cause| {
            self.stop(InputCaptureStop::Read(std::sync::Arc::new(
                HandlerExecutionDenial::new(cause),
            )))
        })
    }
}
