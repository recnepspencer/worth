//! A fresh ordinary request retains the original funding through every outcome.
mod action;
mod funding;
mod source;
pub use source::{
    WorthQueryPerformedSourceInspectionRequest, WorthQuerySourceInspectionRequest,
    WorthQuerySourceReferenceOnlyInspectionRequest,
};
mod connected;
pub use connected::{
    WorthQueryConnectedInspectionRequest, WorthQueryConnectedReferenceOnlyInspectionRequest,
    WorthQueryEvaluatedInspectionRequest, WorthQueryEvaluatedReferenceOnlyInspectionRequest,
};
mod changed;
pub use changed::{
    WorthQueryChangedInspectionRequest, WorthQueryChangedReferenceOnlyInspectionRequest,
};
pub(super) mod binding;
mod governed;
mod rich;
use super::super::WorthQueryApplicationQueryObservation;
use super::{
    stop::{InspectionCause, WorthQueryApplicationInspectionDenial},
    WorthQueryObservedApplicationInspection,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
pub use rich::WorthQueryMaterializedInspectionRequest;
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
        WorthQueryReferenceOnlyInspectionRequest(
            self.0.with_allocation_policy_core(allocation_policy),
        )
    }

    pub fn execute(
        self,
    ) -> Result<WorthQueryObservedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    {
        self.0.execute_with(|_, _, _, _, _| Ok(None))
    }
}

impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    fn with_allocation_policy_core<'funding>(
        self,
        allocation_policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryApplicationInspectionRequest<'funding, 'p, 'r, 'o, Schema, Binding>
    where
        'a: 'funding,
    {
        WorthQueryApplicationInspectionRequest {
            runtime: self.runtime,
            external: self.external,
            request: self.request,
            original: self.original,
            allocation_policy,
            binding: PhantomData,
        }
    }
}

impl<'o, Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>
    WorthQueryApplicationInspectionRequest<'_, '_, '_, 'o, Schema, Binding>
{
    fn open<T>(
        &self,
        body: impl FnOnce(&ExecutionRequest<'_, '_>) -> Result<T, InspectionCause>,
    ) -> Result<T, InspectionCause> {
        funding::open(self.runtime, self.request, self.allocation_policy, body)
    }
}
