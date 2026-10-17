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
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    action: impl FnOnce(
        &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, ProgramRequiredMutationBinding>,
    ) -> Output,
) -> Output {
    let mut output = None;
    let sealed = try_with_reader(phase_custody, world, request, |reader| {
        output = Some(action(reader))
    });
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
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    action: impl FnOnce(
        &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, ProgramRequiredMutationBinding>,
    ) -> Output,
) -> Result<Output, crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial> {
    let principal = authenticated_principal(&(phase_custody).test_host_request(), world, request);
    let account = resolved_account(&(phase_custody).test_host_request(), world, "open", request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request())
        .authorize_operation(
            phase_custody,
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
    let interruption = std::cell::Cell::new(None);
    let execution = crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution::open(world.application.product_runtime().owner.execution_placement(), admission.publication_request());
    world
        .invariant
        .project_admitted_operation(
            phase_custody,
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
                    &interruption,
                );
                action(&mut decision)
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map(|completed| completed.into_parts().0)
}
