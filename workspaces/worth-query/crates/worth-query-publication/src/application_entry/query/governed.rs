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
    WorthQueryApplicationProjection, WorthQueryApplicationQueryAccessContext,
    WorthQueryApprovedElevation, WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};

use crate::domain_computation::{publish_application_result, WorthQueryPublishedApplicationResult};

use super::{WorthQueryApplicationQueryRequest, WorthQueryApplicationRequestQueryDenial};

type Binding<Schema, Intent> = <Intent as ApplicationQueryIntent<Schema>>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding<Schema>>::Scope;

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
    pub fn execute_governed<Capability, Operation, Input>(
        self,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<Schema, Intent>, QueryResult<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.execute_with_approval(None, capability, operation, input)
    }

    pub fn execute_approved<Capability, Operation, Input>(
        self,
        approved: &WorthQueryApprovedElevation,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<Schema, Intent>, QueryResult<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        self.execute_with_approval(Some(approved), capability, operation, input)
    }

    fn execute_with_approval<Capability, Operation, Input>(
        self,
        approved: Option<&WorthQueryApprovedElevation>,
        capability: ApplicationCapabilityRef<Schema, Capability>,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        input: Input,
    ) -> Result<
        WorthQueryPublishedApplicationResult<Query<Schema, Intent>, QueryResult<Schema, Intent>>,
        WorthQueryApplicationRequestQueryDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope<Schema, Intent>>
            + 'static,
    {
        if self.retained.is_some() {
            return Err(WorthQueryApplicationRequestQueryDenial::RequestMode);
        }
        let parameters = self.intent.parameters();
        let scope_binding = self.intent.into_scope();
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Intent::Binding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let limits = match self.limits {
            Some((results, work)) => binding
                .limits()
                .narrow(results, work)
                .map_err(WorthQueryApplicationRequestQueryDenial::Limit)?,
            None => binding.limits(),
        };
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
        let capability_access = match approved {
            Some(approved) => selected.admit_approved_elevation_access(
                approved,
                &principal,
                &installed_capability,
                input,
                self.scope,
            ),
            None => selected.admit_capability_access(
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
            limits.maximum_work(),
            self.scope,
        );
        let plan = selected
            .admit_governed_application_query(
                binding.query(),
                &access,
                capability_access,
                parameters,
                controls,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        let result = self
            .application
            .execute_application_query_one_shot(plan)
            .map_err(WorthQueryApplicationRequestQueryDenial::Execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}
