//! Fresh current governance precedes canonical first-page admission.
use super::*;
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
    pub(super) fn page_with_governance<Capability, Operation, Input>(
        self,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        page_width: NonZeroUsize,
    ) -> Result<Page<Schema, Intent>, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        let application = self.application;
        let request_scope = self.scope;
        application
            .with_application_advancement(request_scope, |phase| {
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
                self.application
                    .resolve_application_query_limits(binding.limits())
                    .narrow(page_width)
                    .map_err(WorthQueryApplicationRequestQueryDenial::Limit)?;
                let selected = self
                    .application
                    .on_branch(self.branch)
                    .select(&phase)
                    .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
                let installed_capability = self
                    .application
                    .installed_schema()
                    .capability(capability, operation)
                    .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityInstallation)?;
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        binding.principal_binding(),
                        self.principal,
                        self.scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .map_err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution)?;
                let capability_access = match approved {
                    Some(approved) => selected.admit_approved_elevation_access(
                        &phase,
                        approved,
                        &principal,
                        &installed_capability,
                        input,
                        self.scope,
                    ),
                    None => selected.admit_capability_access(
                        &phase,
                        &principal,
                        &installed_capability,
                        input,
                        self.scope,
                    ),
                }
                .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityAdmission)?;
                let (scope_field, scope_value) =
                    scope_binding.into_field_parts(principal.principal_identity());
                let scope = selected
                    .resolve_entity(
                        &phase,
                        scope_field,
                        scope_value,
                        self.scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .map_err(WorthQueryApplicationRequestQueryDenial::ScopeResolution)?;
                let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
                let query = binding.into_query();
                let controls = WorthQueryProductQueryControls::new(page_width, self.scope);
                let plan = selected
                    .admit_governed_application_query_continuation(
                        &phase,
                        &query,
                        &access,
                        capability_access,
                        parameters,
                        controls,
                    )
                    .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
                self.application
                    .execute_application_query_continuation_page(&phase, plan)
                    .map_err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution)
            })
            .map_err(WorthQueryApplicationRequestQueryDenial::ExecutionRequest)?
    }
}
