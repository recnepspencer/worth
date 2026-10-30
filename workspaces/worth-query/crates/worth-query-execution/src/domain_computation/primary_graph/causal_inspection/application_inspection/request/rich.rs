//! A new rich request obtains new metadata and diagnostic access; no published report grants it.
use super::super::{
    result::WorthQueryMaterializedApplicationInspection,
    rich::WorthQueryApplicationMaterializationRequest,
};
use super::*;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess as Access;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationQueryScopeBinding,
};
type Scope<Schema, Binding> =
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<
        Schema,
    >>::Scope;
pub struct WorthQueryMaterializedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding> {
    request: WorthQueryApplicationInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>,
    detail: WorthQueryApplicationMaterializationRequest,
}
impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryObservedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    pub fn materialize(
        self,
        detail: WorthQueryApplicationMaterializationRequest,
    ) -> WorthQueryMaterializedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding> {
        WorthQueryMaterializedInspectionRequest {
            request: self.0,
            detail,
        }
    }
}
impl<'a, 'p, 'r, 'o, Schema, Binding>
    WorthQueryMaterializedInspectionRequest<'a, 'p, 'r, 'o, Schema, Binding>
{
    pub fn with_allocation_policy<'funding>(
        self,
        allocation_policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryMaterializedInspectionRequest<'funding, 'p, 'r, 'o, Schema, Binding>
    where
        'a: 'funding,
    {
        WorthQueryMaterializedInspectionRequest {
            request: self.request.with_allocation_policy_core(allocation_policy),
            detail: self.detail,
        }
    }
}
impl<'o, Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>
    WorthQueryMaterializedInspectionRequest<'_, '_, '_, 'o, Schema, Binding>
{
    pub fn execute_with_diagnostic<DC, DO, DI>(
        self,
        diagnostic: Access<Schema, DC, DO, DI>,
    ) -> Result<
        WorthQueryMaterializedApplicationInspection<'o>,
        WorthQueryApplicationInspectionDenial,
    >
    where
        DI: ApplicationCapabilityRequest<Schema, DC, Scope = Scope<Schema, Binding>>,
    {
        let runtime = self.request.runtime;
        let request = self.request.request;
        self.request
            .execute_rich_with(self.detail, |e, q, p, s, m| {
                governed::prepare_capability(runtime, request, e, q, p, s, diagnostic, m)
                    .map(|diagnostic| (None, diagnostic))
            })
    }
    pub fn execute_governed<MC, MO, MI, DC, DO, DI>(
        self,
        metadata: Access<Schema, MC, MO, MI>,
        diagnostic: Access<Schema, DC, DO, DI>,
    ) -> Result<
        WorthQueryMaterializedApplicationInspection<'o>,
        WorthQueryApplicationInspectionDenial,
    >
    where
        MI: ApplicationCapabilityRequest<Schema, MC, Scope = Scope<Schema, Binding>>,
        DI: ApplicationCapabilityRequest<Schema, DC, Scope = Scope<Schema, Binding>>,
    {
        let runtime = self.request.runtime;
        let request = self.request.request;
        self.request
            .execute_rich_with(self.detail, |e, q, p, s, m| {
                let metadata =
                    governed::prepare_capability(runtime, request, e, q, p, s, metadata, m)?;
                let diagnostic =
                    governed::prepare_capability(runtime, request, e, q, p, s, diagnostic, m)?;
                Ok((Some(metadata), diagnostic))
            })
    }
}
