//! Conventional admission keeps all issued operands on the same selection.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation) fn authorize_public_mutation_on_selected<Schema, Binding, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    current: &WorthQueryCurrentRetainedMutationBinding<'_, Schema, Binding>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    principal: &WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,
    request: &WorthQueryRequestScope,
) -> Result<
    WorthQueryAdmittedApplicationOperation<Schema, Binding::Operation, Binding::Input, Scope>,
    SelectedConventionalAdmissionStop,
>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Scope: ApplicationEntityMarkerIdentity<Schema>,
    Binding::ScopeBinding: ApplicationMutationScopeBinding<Schema, Scope = Scope>,
{
    authorize_mutation_products(
        execution,
        runtime,
        SelectedMutationProducts {
            mutation: shared,
            control: None,
            history: None,
        },
        current,
        prepared,
        principal,
        scope,
        request,
    )
}
