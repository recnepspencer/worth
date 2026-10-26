//! The common operation grant ends before the operations' different reads.
use super::super::{schema::*, world::World};
use crate::domain_computation::primary_graph::*;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_schema::*;

pub(in super::super) fn authorize<Operation, Input>(
    world: &World,
    phase: &WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    key: u64,
    operation: ApplicationOperationRef<WalkSchema, Operation, Input>,
    branch: Option<crate::basis::WorthQueryProductBranch>,
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
            .select_product_branch(
                execution,
                world.application.product_runtime().default_branch(),
            )
            .unwrap(),
    };
    let external = crate::domain_computation::primary_graph::tests::fixture::authenticate_external(
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
            NodeKey::reference(),
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
