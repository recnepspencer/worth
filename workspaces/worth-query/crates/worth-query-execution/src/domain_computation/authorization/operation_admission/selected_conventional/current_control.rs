//! Current installed principal/scope authorize a distinct mutation selection.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation) fn authorize_current_control_mutation_on_selected<
    Schema,
    Binding,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &dyn crate::domain_computation::authorization::host::AuthorizationHost<Schema>,
    mutation: &SharedSelectedProductOperation<'_, Schema>,
    control: &SharedSelectedProductOperation<'_, Schema>,
    current: &WorthQueryCurrentRetainedMutationBinding<'_, Schema, Binding>,
    prepared: &WorthQueryPreparedApplicationOperationGraphWork,
    principal: &WorthQueryIssuedSelectedPrincipal<
        '_,
        '_,
        Schema,
        Binding::Principal,
        Binding::PrincipalIdentity,
    >,
    control_scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,
    mutation_scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,
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
    use SelectedConventionalAdmissionStop as Stop;
    // Compare actual issued selections and complete entity identity, including
    // its generation. Matching payload or a reusable numeric slot is insufficient.
    if !std::ptr::addr_eq(control.selected().application(), runtime)
        || !std::ptr::addr_eq(mutation.selected().application(), runtime)
        || !std::ptr::eq(mutation_scope.selected(), mutation.selected())
        || !mutation_scope.request().same_request(request)
        || control_scope.scope().entity_id() != mutation_scope.scope().entity_id()
        || control_scope.scope().entity_kind() != mutation_scope.scope().entity_kind()
    {
        return Err(Stop::ForeignIssuedSource);
    }
    authorize_mutation_products(
        execution,
        runtime,
        SelectedMutationProducts {
            mutation,
            control: Some(control),
        },
        current,
        prepared,
        principal,
        control_scope,
        request,
    )
}
