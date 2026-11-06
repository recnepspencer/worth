//! Resolve real authentication and authorization before building the effect program.
use crate::domain_computation::primary_graph::tests::application_attempt::checked_invariant_hooks::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::checked_invariant_hooks::world::World;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

pub fn prepare(
    world: &World,
    scope: &WorthQueryRequestScope,
) -> WorthQueryApplicationEffectProgram<HookSchema, ChangeStatus, ChangeStatusInput, Account> {
    world
        .application
        .with_host_advancement(|phase| {
            let host_request = phase.test_host_request_for_test();
            let execution = host_request
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            let selected = world
                .application
                .select_product_branch_for_test(
                    &execution,
                    world.application.product_runtime().default_branch_for_test(),
                )
                .unwrap();
            let external =
                worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
                    world.application.installed_schema(),
                    "alice",
                    std::time::Duration::from_secs(60),
                    scope,
                );
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = world
                .application
                .installed_schema()
                .installed_operation(ChangeStatus::reference())
                .unwrap();
            let admission = selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    scope,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, projected| {
                        reader
                            .require_decision_field(projected, AccountStatus::at("AccountStatus"))
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let mut effects = reads
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program();
            let account = effects.existing_entity(&account).unwrap();
            effects
                .write_field(
                    &account,
                    AccountStatus::at("AccountStatus"),
                    "checked-rule".to_owned(),
                )
                .unwrap();
            effects.finish().unwrap()
        })
        .unwrap()
}
