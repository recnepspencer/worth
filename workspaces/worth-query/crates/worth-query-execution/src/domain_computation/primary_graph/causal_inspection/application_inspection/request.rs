//! A fresh ordinary request retains the original funding through every outcome.
mod action;
pub(super) mod binding;
mod governed;
use super::super::WorthQueryApplicationQueryObservation;
use super::{
    result::PreparedObservedInspection,
    stop::{InspectionCause, WorthQueryApplicationInspectionDenial},
    WorthQueryObservedApplicationInspection,
};
use crate::domain_computation::primary_graph::{
    application_contribution::WorthQueryAdvancementDenial, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::marker::PhantomData;
use worth_execution::{ExecutionAllocationPolicy, ExecutionMemoryReservation, ExecutionRequest};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchema;

/// Borrowed original observation and fresh request; construction reads no graph.
pub struct WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &'r WorthQueryRequestScope,
    original: &'o WorthQueryApplicationQueryObservation,
    allocation_policy: ExecutionAllocationPolicy<'a, 'a>,
    binding: PhantomData<fn() -> Binding>,
}

/// Selects the meaning established by one genuine completed original query.
pub struct WorthQueryObservedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>(
    WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>,
);

/// Admits the Query reference and fixed metadata without lower-family claims.
pub struct WorthQueryReferenceOnlyInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>(
    WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>,
);

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Inspects the current occurrence of the original branch, with fresh access.
    pub fn inspect_application_observation<'a, 'p, 'r, 'o, Binding>(
        &'a self,
        external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &'r WorthQueryRequestScope,
        original: &'o WorthQueryApplicationQueryObservation,
    ) -> WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        WorthQueryApplicationInspectionRequest {
            runtime: self,
            external,
            request,
            original,
            allocation_policy: ExecutionAllocationPolicy::SystemAllocation,
            binding: PhantomData,
        }
    }
}

impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    pub fn why_observed(
        self,
    ) -> WorthQueryObservedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding> {
        WorthQueryObservedInspectionRequest(self)
    }
}

impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryObservedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    pub fn reference_only(
        self,
    ) -> WorthQueryReferenceOnlyInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding> {
        WorthQueryReferenceOnlyInspectionRequest(self.0)
    }
}

impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryReferenceOnlyInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    /// Funds the same fresh request under a real caller lease. The default
    /// SystemAllocation delegates to the installed World's execution policy.
    pub fn with_allocation_policy<'funding>(
        self,
        allocation_policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryReferenceOnlyInspectionRequest<'funding, 'p, 'r, 'o, Schema, Binding>
    where
        'a: 'funding,
    {
        WorthQueryReferenceOnlyInspectionRequest(WorthQueryApplicationInspectionRequest {
            runtime: self.0.runtime,
            external: self.0.external,
            request: self.0.request,
            original: self.0.original,
            allocation_policy,
            binding: PhantomData,
        })
    }

    pub fn execute(
        self,
    ) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    {
        self.0.execute_with(|_, _, _, _, _| Ok(None))
    }
}

impl<'o, Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>
    WorthQueryApplicationInspectionRequest<'_, '_, '_, 'o, Schema, Binding>
{
    fn open<T>(
        &self,
        body: impl FnOnce(&ExecutionRequest<'_, '_>) -> Result<T, InspectionCause>,
    ) -> Result<T, InspectionCause> {
        let mut completed = None;
        let handoff = self.runtime.with_funded_application_advancement(
            self.request,
            self.allocation_policy,
            |phase| {
                completed = Some(
                    match phase.checked_execution_request_for(&self.runtime.product_runtime) {
                        Ok(execution) => body(&execution),
                        Err(cause) => Err(InspectionCause::Advancement(cause)),
                    },
                );
            },
        );
        match (handoff, completed) {
            (Ok(()), Some(result)) => result,
            (Err(WorthQueryAdvancementDenial::NestedStopped), Some(Err(cause))) => Err(cause),
            (Err(cause), _) => Err(InspectionCause::Advancement(cause)),
            (Ok(()), None) => unreachable!("successful original advancement invokes inspection"),
        }
    }

    fn finish(
        result: Result<PreparedObservedInspection<'o>, InspectionCause>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    {
        match result {
            Ok(prepared) => Ok(prepared.fund(memory)),
            Err(cause) => Err(WorthQueryApplicationInspectionDenial::new(cause, memory)),
        }
    }
}
