//! Resolve real authentication and authorization before building the effect program.
use super::schema::*;
use super::world::World;
use crate::domain_computation::primary_graph::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

pub(super) fn prepare(
    world: &World,
    scope: &WorthQueryRequestScope,
    status: String,
) -> WorthQueryApplicationEffectProgram<RefusalSchema, ChangeStatus, ChangeStatusInput, Account> {
    authorized(
        world,
        scope,
        ChangeStatus::reference(),
        |effects, account| {
            effects
                .write_field(account, AccountStatus::reference(), status)
                .unwrap();
        },
    )
}

/// Deletes the authorized account.
pub(super) fn close(
    world: &World,
    scope: &WorthQueryRequestScope,
) -> WorthQueryApplicationEffectProgram<RefusalSchema, CloseAccount, CloseAccountInput, Account> {
    authorized(
        world,
        scope,
        CloseAccount::reference(),
        |effects, account| {
            effects
                .delete_entity(Account::reference(), account)
                .unwrap();
        },
    )
}

fn authorized<Operation, Input>(
    world: &World,
    scope: &WorthQueryRequestScope,
    operation: worth_query_declaration::facade::application_schema::ApplicationOperationRef<
        RefusalSchema,
        Operation,
        Input,
    >,
    effect: impl FnOnce(
        &mut WorthQueryApplicationEffectProgramBuilder<RefusalSchema, Operation, Input, Account>,
        &WorthQueryApplicationEffectEntity<RefusalSchema, Account>,
    ),
) -> WorthQueryApplicationEffectProgram<RefusalSchema, Operation, Input, Account>
where
    Operation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                RefusalSchema,
            > + 'static,
    Operation::InputBinding:
        worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
            Value = Input,
        >,
    Input: 'static,
    Account: worth_query_installation::facade::OperationReads<Operation>,
    AccountStatus: worth_query_installation::facade::OperationReads<Operation>,
{
    world
        .application
        .with_host_advancement(|phase| {
            let host_request = phase.test_host_request();
            let execution = host_request
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let selected = world
                .application
                .select_product_branch(
                    &execution,
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            let external =
                crate::domain_computation::primary_graph::tests::fixture::authenticate_external(
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
                    AccountStatus::reference(),
                    "open".to_owned(),
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = world
                .application
                .installed_schema()
                .installed_operation(operation)
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
                            .require_decision_entity(projected, Account::reference())
                            .unwrap();
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
            effect(&mut effects, &account);
            effects.finish().unwrap()
        })
        .unwrap()
}
