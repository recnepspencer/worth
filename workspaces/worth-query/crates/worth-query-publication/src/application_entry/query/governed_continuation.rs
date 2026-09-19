use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationContinuationPageResult,
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryContinuation, WorthQueryApplicationQueryResumeControls,
    WorthQueryApprovedElevation, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};

use super::{WorthQueryApplicationQueryRequest, WorthQueryApplicationRequestQueryDenial};

type Binding<Schema, Intent> = <Intent as ApplicationQueryIntent<Schema>>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type Parameters<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Principal;
type PrincipalIdentity<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding<Schema>>::Scope;

type Page<Schema, Intent> = WorthQueryApplicationContinuationPageResult<
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Scope<Schema, Intent>,
>;
type Continuation<Schema, Intent> = WorthQueryApplicationQueryContinuation<
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Scope<Schema, Intent>,
>;
type Plan<'a, Schema, Intent> = WorthQueryAdmittedApplicationQueryPlan<
    'a,
    Schema,
    Query<Schema, Intent>,
    Parameters<Schema, Intent>,
    QueryResult<Schema, Intent>,
    Principal<Schema, Intent>,
    PrincipalIdentity<Schema, Intent>,
    Scope<Schema, Intent>,
>;

impl<Schema, Intent> WorthQueryApplicationQueryRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    <Intent::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub fn page_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        page_width: NonZeroUsize,
        maximum_work: NonZeroUsize,
    ) -> Result<Page<Schema, Intent>, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        if self.retained.is_some() || self.limits.is_some() {
            return Err(WorthQueryApplicationRequestQueryDenial::RequestMode);
        }
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
        let installed_capability = self
            .application
            .installed_schema()
            .capability(capability, operation)
            .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityInstallation)?;
        let principal = selected
            .resolve_authenticated_principal(
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution)?;
        let capability_access = selected
            .admit_approved_elevation_access(
                approved,
                &principal,
                &installed_capability,
                input,
                self.scope,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityAdmission)?;
        let (scope_field, scope_value) =
            scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::ScopeResolution)?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let query = binding.into_query();
        let controls = WorthQueryProductQueryControls::new(page_width, maximum_work, self.scope);
        let plan = selected
            .admit_governed_application_query_continuation(
                &query,
                &access,
                capability_access,
                parameters,
                controls,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        self.application
            .execute_application_query_continuation_page(plan)
            .map_err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn readmit_resume_approved<Capability, Operation, Input, Output>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        continuation: Continuation<Schema, Intent>,
        page_width: NonZeroUsize,
        maximum_work: NonZeroUsize,
        after_readmission: impl for<'admitted> FnOnce(
            &'admitted WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            Plan<'admitted, Schema, Intent>,
        ) -> Output,
    ) -> Result<Output, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        if self.retained.is_some() || self.limits.is_some() {
            return Err(WorthQueryApplicationRequestQueryDenial::RequestMode);
        }
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let selected = self
            .application
            .on_branch(self.branch)
            .select()
            .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
        let installed_capability = self
            .application
            .installed_schema()
            .capability(capability, operation)
            .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityInstallation)?;
        let principal = selected
            .resolve_authenticated_principal(
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution)?;
        let capability_access = selected
            .admit_approved_elevation_access(
                approved,
                &principal,
                &installed_capability,
                input,
                self.scope,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityAdmission)?;
        let (scope_field, scope_value) =
            scope_binding.into_field_parts(principal.principal_identity());
        let scope = selected
            .resolve_entity(
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::ScopeResolution)?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let query = binding.into_query();
        let controls =
            WorthQueryApplicationQueryResumeControls::new(page_width, maximum_work, self.scope);
        let plan = self
            .application
            .readmit_governed_application_query_continuation(
                &query,
                &access,
                capability_access,
                parameters,
                continuation,
                controls,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        Ok(after_readmission(self.application, plan))
    }
}
