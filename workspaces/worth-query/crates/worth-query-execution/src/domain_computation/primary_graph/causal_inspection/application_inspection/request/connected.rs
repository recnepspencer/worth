//! A typed original source and query demand independent current metadata/diagnostics.
use super::super::result::WorthQueryConnectedApplicationInspection;
use super::*;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess as Access;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::ApplicationSourceDisclosureBinding,
    application_query::ApplicationQueryScopeBinding,
};
mod action;

type QueryScope<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<
        Schema,
    >>::Scope;
type SourceScope<Schema,Disclosure>=<<<Disclosure as ApplicationSourceDisclosureBinding<Schema>>::Disclosure as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;

pub struct WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure> {
    request: WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>,
    source: &'s crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    disclosure: PhantomData<fn() -> Disclosure>,
}
pub struct WorthQueryConnectedReferenceOnlyInspectionRequest<
    'a,
    'p,
    'r,
    'o,
    's,
    Schema,
    Binding,
    Disclosure,
>(WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>);
impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryObservedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    /// Select original source interpretation without traversing evidence or granting access.
    pub fn caused_by<'s, Disclosure: ApplicationSourceDisclosureBinding<Schema>>(
        self,
        source: &'s crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    where
        Schema: ApplicationSchema,
    {
        WorthQueryConnectedInspectionRequest {
            request: self.0,
            source,
            disclosure: PhantomData,
        }
    }
}
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryConnectedInspectionRequest<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
{
    pub fn reference_only(
        self,
    ) -> WorthQueryConnectedReferenceOnlyInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    > {
        WorthQueryConnectedReferenceOnlyInspectionRequest(self)
    }
}
impl<'a, 'p, 'r, 'o, 's, Schema, Binding, Disclosure>
    WorthQueryConnectedReferenceOnlyInspectionRequest<
        'a,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    >
{
    pub fn with_allocation_policy<'funding>(
        self,
        allocation_policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryConnectedReferenceOnlyInspectionRequest<
        'funding,
        'p,
        'r,
        'o,
        's,
        Schema,
        Binding,
        Disclosure,
    >
    where
        'a: 'funding,
    {
        WorthQueryConnectedReferenceOnlyInspectionRequest(WorthQueryConnectedInspectionRequest {
            request: WorthQueryApplicationInspectionRequest {
                runtime: self.0.request.runtime,
                external: self.0.request.external,
                request: self.0.request.request,
                original: self.0.request.original,
                allocation_policy,
                binding: PhantomData,
            },
            source: self.0.source,
            disclosure: PhantomData,
        })
    }
}

impl<'o, Schema, Binding, Disclosure>
    WorthQueryConnectedReferenceOnlyInspectionRequest<
        '_,
        '_,
        '_,
        'o,
        '_,
        Schema,
        Binding,
        Disclosure,
    >
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
    Disclosure: ApplicationSourceDisclosureBinding<Schema>,
{
    pub fn execute_with_diagnostics<QDC, QDO, QDI, SDC, SDO, SDI>(
        self,
        qd: Access<Schema, QDC, QDO, QDI>,
        sd: Access<Schema, SDC, SDO, SDI>,
    ) -> Result<WorthQueryConnectedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        QDI: ApplicationCapabilityRequest<Schema, QDC, Scope = QueryScope<Schema, Binding>>,
        SDI: ApplicationCapabilityRequest<Schema, SDC, Scope = SourceScope<Schema, Disclosure>>,
    {
        let runtime = self.0.request.runtime;
        let request = self.0.request.request;
        self.0
            .execute_with(|execution, query, source, principal, scope, memory| {
                let qd = governed::prepare_capability(
                    runtime, request, execution, query, principal, scope, qd, memory,
                )?;
                let sd = governed::prepare_capability(
                    runtime, request, execution, source, principal, scope, sd, memory,
                )?;
                Ok(action::ConnectionPolicies {
                    query_metadata: None,
                    query_diagnostic: Some(qd),
                    source_metadata: None,
                    source_diagnostic: sd,
                })
            })
    }
    pub fn execute_governed<QMC, QMO, QMI, QDC, QDO, QDI, SMC, SMO, SMI, SDC, SDO, SDI>(
        self,
        qm: Access<Schema, QMC, QMO, QMI>,
        qd: Access<Schema, QDC, QDO, QDI>,
        sm: Access<Schema, SMC, SMO, SMI>,
        sd: Access<Schema, SDC, SDO, SDI>,
    ) -> Result<WorthQueryConnectedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial>
    where
        QMI: ApplicationCapabilityRequest<Schema, QMC, Scope = QueryScope<Schema, Binding>>,
        QDI: ApplicationCapabilityRequest<Schema, QDC, Scope = QueryScope<Schema, Binding>>,
        SMI: ApplicationCapabilityRequest<Schema, SMC, Scope = SourceScope<Schema, Disclosure>>,
        SDI: ApplicationCapabilityRequest<Schema, SDC, Scope = SourceScope<Schema, Disclosure>>,
    {
        let runtime = self.0.request.runtime;
        let request = self.0.request.request;
        self.0
            .execute_with(|execution, query, source, principal, scope, memory| {
                let qm = governed::prepare_capability(
                    runtime, request, execution, query, principal, scope, qm, memory,
                )?;
                let qd = governed::prepare_capability(
                    runtime, request, execution, query, principal, scope, qd, memory,
                )?;
                let sm = governed::prepare_capability(
                    runtime, request, execution, source, principal, scope, sm, memory,
                )?;
                let sd = governed::prepare_capability(
                    runtime, request, execution, source, principal, scope, sd, memory,
                )?;
                Ok(action::ConnectionPolicies {
                    query_metadata: Some(qm),
                    query_diagnostic: Some(qd),
                    source_metadata: Some(sm),
                    source_diagnostic: sd,
                })
            })
    }
}
