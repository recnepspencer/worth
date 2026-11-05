//! Admitted programs built from a selected product.
use super::*;

pub fn admitted_program_from_selected(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    options: ProgramOptions<'_>,
) -> Program {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = selected
        .authorize_operation(
            phase_custody,
            principal,
            account,
            &operation,
            options.preconditions,
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountStatus::reference())
                    .unwrap();
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            phase_custody,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(account).unwrap();
    effects
        .write_field(
            &account,
            AccountStatus::reference(),
            options.replacement.to_owned(),
        )
        .unwrap();
    for payload in options.emissions {
        effects
            .emit(
                crate::domain_computation::primary_graph::tests::fixture::AccountActivityEffect::reference(),
                payload.to_owned(),
            )
            .unwrap();
    }
    effects.finish().unwrap()
}
