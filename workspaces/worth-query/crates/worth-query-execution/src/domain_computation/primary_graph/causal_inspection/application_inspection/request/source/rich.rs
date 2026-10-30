//! Source-only richness requires a new request and its current diagnostic capability.
use super::super::super::{
    result::WorthQueryMaterializedPerformedSourceInspection,
    rich::WorthQueryApplicationMaterializationRequest,
};
use super::*;
type Scope<Schema,Disclosure>=<<<Disclosure as ApplicationSourceDisclosureBinding<Schema>>::Disclosure as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;
pub struct WorthQueryMaterializedSourceInspectionRequest<'a, 'p, 'r, 's, Schema, Disclosure> {
    request: WorthQuerySourceInspectionRequest<'a, 'p, 'r, 's, Schema, Disclosure>,
    detail: WorthQueryApplicationMaterializationRequest,
}
impl<'a, 'p, 'r, 's, Schema, Disclosure>
    WorthQueryPerformedSourceInspectionRequest<'a, 'p, 'r, 's, Schema, Disclosure>
{
    pub fn materialize(
        self,
        detail: WorthQueryApplicationMaterializationRequest,
    ) -> WorthQueryMaterializedSourceInspectionRequest<'a, 'p, 'r, 's, Schema, Disclosure> {
        WorthQueryMaterializedSourceInspectionRequest {
            request: self.0,
            detail,
        }
    }
}
impl<'a, 'p, 'r, 's, Schema, Disclosure>
    WorthQueryMaterializedSourceInspectionRequest<'a, 'p, 'r, 's, Schema, Disclosure>
{
    pub fn with_allocation_policy<'funding>(
        self,
        policy: ExecutionAllocationPolicy<'funding, 'funding>,
    ) -> WorthQueryMaterializedSourceInspectionRequest<'funding, 'p, 'r, 's, Schema, Disclosure>
    where
        'a: 'funding,
    {
        WorthQueryMaterializedSourceInspectionRequest {
            request: self.request.with_allocation_policy_core(policy),
            detail: self.detail,
        }
    }
}
impl<Schema: ApplicationSchema, Disclosure: ApplicationSourceDisclosureBinding<Schema>>
    WorthQueryMaterializedSourceInspectionRequest<'_, '_, '_, '_, Schema, Disclosure>
{
    pub fn execute_with_diagnostic<C, O, I>(
        self,
        diagnostic: WorthQueryAdmittedApplicationCapabilityAccess<Schema, C, O, I>,
    ) -> Result<
        WorthQueryMaterializedPerformedSourceInspection,
        WorthQueryApplicationInspectionDenial,
    >
    where
        I: ApplicationCapabilityRequest<Schema, C, Scope = Scope<Schema, Disclosure>>,
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
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, MC, MO, MI>,
        diagnostic: WorthQueryAdmittedApplicationCapabilityAccess<Schema, DC, DO, DI>,
    ) -> Result<
        WorthQueryMaterializedPerformedSourceInspection,
        WorthQueryApplicationInspectionDenial,
    >
    where
        MI: ApplicationCapabilityRequest<Schema, MC, Scope = Scope<Schema, Disclosure>>,
        DI: ApplicationCapabilityRequest<Schema, DC, Scope = Scope<Schema, Disclosure>>,
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
