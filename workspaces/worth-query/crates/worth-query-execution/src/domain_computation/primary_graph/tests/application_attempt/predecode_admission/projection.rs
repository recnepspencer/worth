//! Same production DecisionReader construction over a genuinely admitted scope.
use super::super::{authenticated_principal, resolved_account};
use crate::domain_computation::primary_graph::{
    handler::DecisionContextUse,
    tests::fixture::{
        AuthorizationWorld, IdentityExecutionSchema, ProgramRequiredInput,
        ProgramRequiredMutationBinding, ProgramRequiredOperation,
    },
    DecisionReader,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;

pub(super) fn with_reader<Output>(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    action: impl FnOnce(
        &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, ProgramRequiredMutationBinding>,
    ) -> Output,
) -> Output {
    let principal = authenticated_principal(world, request);
    let account = resolved_account(world, "open", request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            request,
        )
        .unwrap();
    let input = ProgramRequiredInput::new("open");
    let key = "predecode-installed".to_owned();
    let identities = ApplicationMutationIdentities::<
        IdentityExecutionSchema,
        ProgramRequiredMutationBinding,
    >::encode(&key, &input)
    .unwrap();
    let affinity = admission.operation_scope_binding().clone();
    let context = std::cell::Cell::new(DecisionContextUse::default());
    world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, scope| {
                let mut decision = DecisionReader::new(
                    reader,
                    scope,
                    principal.principal_identity(),
                    &affinity,
                    &identities,
                    admission.publication_request(),
                    &context,
                );
                action(&mut decision)
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts()
        .0
}
