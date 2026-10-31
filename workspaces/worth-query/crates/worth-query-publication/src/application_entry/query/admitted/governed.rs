//! One actual capability/approval admission owner for governed Query consumers.
use super::*;
use worth_query_declaration::facade::{
    application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationOperationRef},
};
use worth_query_execution::facade::primary_graph::WorthQueryApprovedElevation;

type Scope<S, I> =
    <<I as ApplicationQueryIntent<S>>::Binding as ApplicationQueryBinding<S>>::ScopeBinding;
type ScopeEntity<S, I> = <Scope<S, I> as ApplicationQueryScopeBinding<S>>::Scope;

impl<Schema, Intent> WorthQueryApplicationQueryRequest<'_, '_, '_, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    Scope<Schema, Intent>: ApplicationQueryScopeResolution<
        Schema,
        <Intent::Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
    >,
{
    pub(in crate::application_entry::query) fn with_admitted_governed_query<
        Capability,
        Operation,
        Input,
        T,
    >(
        self,
        phase: &AdvancementPhase<'_>,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
        perform: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            &WorthQueryInstalledApplicationQueryBinding<Schema, Intent::Binding>,
            &WorthQueryAuthenticatedExternalPrincipal<Schema>,
            RequestPlan<'_, Schema, Intent::Binding>,
        ) -> Result<T, WorthQueryApplicationRequestQueryDenial>,
    ) -> Result<T, WorthQueryApplicationRequestQueryDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = ScopeEntity<Schema, Intent>>
            + 'static,
    {
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let limits = match self.limits {
            Some(results) => self
                .application
                .resolve_application_query_limits(binding.limits())
                .narrow(results)
                .map_err(WorthQueryApplicationRequestQueryDenial::Limit)?,
            None => self
                .application
                .resolve_application_query_limits(binding.limits()),
        };
        let selected = self
            .application
            .on_branch(self.branch)
            .select(phase)
            .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
        let retained = self
            .retained
            .as_ref()
            .map(|observation| {
                self.application
                    .select_application_read_observation(phase, observation)
                    .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)
            })
            .transpose()?;
        let installed_capability = self
            .application
            .installed_schema()
            .capability(capability, operation)
            .map_err(WorthQueryApplicationRequestQueryDenial::CapabilityInstallation)?;
        let principal = selected
            .resolve_authenticated_principal(
                phase,
                binding.principal_binding(),
                self.principal,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution)?;
        let capability_access = match approved {
            Some(approved) => selected.admit_approved_elevation_access(
                phase,
                approved,
                &principal,
                &installed_capability,
                input,
                self.scope,
            ),
            None => selected.admit_capability_access(
                phase,
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
                phase,
                scope_field,
                scope_value,
                self.scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::ScopeResolution)?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let query_maximum = NonZeroUsize::new(binding.query().read_graph().maximum_result_count())
            .unwrap_or(NonZeroUsize::MAX);
        let controls = WorthQueryProductQueryControls::new(
            limits.maximum_results().min(query_maximum),
            self.scope,
        );
        let plan = match retained {
            Some(retained) => selected.admit_retained_governed_application_query(
                phase,
                retained,
                binding.query(),
                &access,
                capability_access,
                parameters,
                controls,
            ),
            None => selected.admit_governed_application_query(
                phase,
                binding.query(),
                &access,
                capability_access,
                parameters,
                controls,
            ),
        }
        .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        let plan = if self.retain_projection_for_inspection {
            plan.retain_projection_for_inspection()
        } else {
            plan
        };
        perform(self.application, &binding, self.principal, plan)
    }
}
