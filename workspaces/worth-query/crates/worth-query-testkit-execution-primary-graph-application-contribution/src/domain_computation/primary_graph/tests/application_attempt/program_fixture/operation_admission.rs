//! Admits the fixture operation against its selected product.
use super::*;
pub(in crate::domain_computation::primary_graph) fn admitted_operation(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryAdmittedApplicationOperation<Schema, TouchAccountOperation, Input, Account> {
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    selected
        .authorize_operation(
            phase_custody,
            principal,
            account,
            &operation,
            Preconditions::new(),
            request,
        )
        .unwrap()
}
