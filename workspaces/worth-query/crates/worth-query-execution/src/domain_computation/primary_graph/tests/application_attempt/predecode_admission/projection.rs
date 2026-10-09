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
    let mut output = None;
    let sealed = try_with_reader(world, request, |reader| output = Some(action(reader)));
    if let Err(denial) = sealed {
        let interruption = request
            .interruption()
            .expect("only interrupted custody is refused");
        assert_eq!(
            denial
                .invariant_denial()
                .and_then(|denial| denial.source_retention_interruption()),
            Some(interruption),
        );
    }
    output.expect("the admitted reader callback ran")
}

pub(super) fn try_with_reader<Output>(
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    action: impl FnOnce(
        &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, ProgramRequiredMutationBinding>,
    ) -> Output,
) -> Result<Output, crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial> {
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
    let execution = crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution::open(world.application.product_runtime().owner.execution_placement(), admission.publication_request());
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
                    &execution,
                    &context,
                );
                action(&mut decision)
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map(|completed| completed.into_parts().0)
}
