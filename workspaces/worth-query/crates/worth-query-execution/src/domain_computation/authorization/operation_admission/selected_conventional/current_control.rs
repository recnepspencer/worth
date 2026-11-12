//! Current installed principal/scope authorize a distinct mutation selection.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation) fn authorize_current_control_mutation_on_selected<
    Schema,
    Binding,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
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
    if !std::ptr::eq(control.selected().application(), runtime)
        || !std::ptr::eq(mutation.selected().application(), runtime)
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
            history: None,
        },
        current,
        prepared,
        principal,
        SelectedMutationScope::Issued(control_scope),
        request,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation) fn authorize_historical_control_mutation_on_selected<
    Schema,
    Binding,
    Scope,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
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
    mutation_scope: &WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,
    history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'_>,
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
    if !std::ptr::eq(control.selected().application(), runtime)
        || !std::ptr::eq(mutation.selected().application(), runtime)
        || !std::ptr::eq(mutation_scope.selected(), mutation.selected())
        || !mutation_scope.request().same_request(request)
        || history.current().observation() != control.selected().product().observation()
        || !history.admits_historical_truth(mutation.selected())
        || !history.admits_request(request)
        || !history.admits_principal(&crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(principal.principal().principal_entity_id()))
    {
        return Err(Stop::ForeignIssuedSource);
    }
    authorize_mutation_products(
        execution,
        runtime,
        SelectedMutationProducts {
            mutation,
            control: Some(control),
            history: Some(history),
        },
        current,
        prepared,
        principal,
        SelectedMutationScope::Issued(mutation_scope),
        request,
    )
}
