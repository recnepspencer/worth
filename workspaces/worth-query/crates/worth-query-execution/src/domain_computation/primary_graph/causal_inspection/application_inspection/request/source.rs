//! A typed original source selects its own independently installed disclosure.
use super::super::result::WorthQueryPerformedSourceInspection;
use super::*;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::ApplicationSourceDisclosureBinding,
    application_query::ApplicationQueryScopeBinding,
};
mod action;

pub struct WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &'r WorthQueryRequestScope,
    source: &'source WorthQueryApplicationCommitReceipt,
    allocation_policy: ExecutionAllocationPolicy<'a, 'a>,
    disclosure: PhantomData<fn() -> Disclosure>,
}
pub struct WorthQueryPerformedSourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>(
    WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>,
);
pub struct WorthQuerySourceReferenceOnlyInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>(
    WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>,
);
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn inspect_application_source<
        'a,
        'p,
        'r,
        'source,
        Disclosure: ApplicationSourceDisclosureBinding<Schema>,
    >(
        &'a self,
        external: &'p WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &'r WorthQueryRequestScope,
        source: &'source WorthQueryApplicationCommitReceipt,
    ) -> WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure> {
        WorthQuerySourceInspectionRequest {
            runtime: self,
            external,
            request,
            source,
            allocation_policy: ExecutionAllocationPolicy::SystemAllocation,
            disclosure: PhantomData,
        }
    }
}
impl<'a, 'p, 'r, 'source, Schema, Disclosure>
    WorthQuerySourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>
{
    pub fn why_performed(
        self,
    ) -> WorthQueryPerformedSourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure> {
        WorthQueryPerformedSourceInspectionRequest(self)
    }
}
impl<'a, 'p, 'r, 'source, Schema, Disclosure>
    WorthQueryPerformedSourceInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>
{
    pub fn reference_only(
        self,
    ) -> WorthQuerySourceReferenceOnlyInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>
    {
        WorthQuerySourceReferenceOnlyInspectionRequest(self.0)
    }
}
impl<
        'a,
        'p,
        'r,
        'source,
        Schema: ApplicationSchema,
        Disclosure: ApplicationSourceDisclosureBinding<Schema>,
    > WorthQuerySourceReferenceOnlyInspectionRequest<'a, 'p, 'r, 'source, Schema, Disclosure>
{
    pub fn with_allocation_policy<'funding>(
        self,
        allocation_policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQuerySourceReferenceOnlyInspectionRequest<'funding, 'p, 'r, 'source, Schema, Disclosure>
    where
        'a: 'funding,
    {
        WorthQuerySourceReferenceOnlyInspectionRequest(WorthQuerySourceInspectionRequest {
            runtime: self.0.runtime,
            external: self.0.external,
            request: self.0.request,
            source: self.0.source,
            allocation_policy,
            disclosure: PhantomData,
        })
    }
    pub fn execute(
        self,
    ) -> Result<WorthQueryPerformedSourceInspection, WorthQueryApplicationInspectionDenial> {
        self.0.execute_with(|_, _, _, _, _| Ok(None))
    }
    pub fn execute_governed<Capability, Operation, Input>(self, metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>) -> Result<WorthQueryPerformedSourceInspection, WorthQueryApplicationInspectionDenial>
    where Input: ApplicationCapabilityRequest<Schema, Capability, Scope = <<Disclosure::Disclosure as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>{
        let runtime = self.0.runtime;
        let request = self.0.request;
        self.0
            .execute_with(|execution, installed, principal, scope, memory| {
                governed::prepare_capability(
                    runtime, request, execution, installed, principal, scope, metadata, memory,
                )
                .map(Some)
            })
    }
}
