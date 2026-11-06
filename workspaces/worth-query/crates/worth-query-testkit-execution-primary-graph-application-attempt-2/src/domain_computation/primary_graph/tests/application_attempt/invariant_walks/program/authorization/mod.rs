//! The common operation grant ends before the operations' different reads.
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::World;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::*;

pub fn authorize<Operation, Input>(
    world: &World,
    phase: &WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    key: u64,
    operation: ApplicationOperationRef<WalkSchema, Operation, Input>,
    branch: Option<worth_query_execution::facade::product::WorthQueryProductBranch>,
) -> (
    WorthQueryAdmittedApplicationOperation<WalkSchema, Operation, Input, Node>,
    WorthQueryApplicationEntityIdentity<WalkSchema, Node>,
)
where
    Operation: ApplicationOperationMarkerIdentity<WalkSchema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: 'static,
{
    let selected = match branch {
        Some(branch) => world.application.on_branch(branch).select(phase).unwrap(),
        None => world
            .application
            .select_product_branch_for_test(
                execution,
                world
                    .application
                    .product_runtime()
                    .default_branch_for_test(),
            )
            .unwrap(),
    };
    let external = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authentication::authenticate_external(
        world.application.installed_schema(),
        "alice",
        std::time::Duration::from_secs(60),
        request,
    );
    let actor = selected
        .resolve_authenticated_principal(
            phase,
            &world.binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let subject = selected
        .resolve_entity(
            phase,
            NodeKey::at("NodeKey"),
            key,
            request,
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
            phase,
            &actor,
            &subject,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    (admission, subject)
}
