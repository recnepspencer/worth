//! Same production DecisionReader construction over a genuinely admitted scope.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredInput;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredMutationBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredOperation;
use worth_query_execution::facade::domain_computation::DecisionReader;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationIdentities;

pub fn with_reader<Output>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
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

pub fn try_with_reader<Output>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
    request: &WorthQueryRequestScope,
    action: impl FnOnce(
        &mut DecisionReader<'_, '_, '_, IdentityExecutionSchema, ProgramRequiredMutationBinding>,
    ) -> Output,
) -> Result<
    Output,
    worth_query_execution::facade::domain_computation::WorthQueryOperationProjectionDenial,
> {
    let principal = authenticated_principal(
        &(phase_custody).test_host_request_for_test(),
        world,
        request,
    );
    let account = resolved_account(
        &(phase_custody).test_host_request_for_test(),
        world,
        "open",
        request,
    );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
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
    world.application.product_runtime().with_execution_placement_for_test(|placement| {
    let execution = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::QueryRequestExecution::open_for_test(placement, admission.publication_request_for_test());
    world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, scope| {
                let mut decision = DecisionReader::new_for_test(
                    reader,
                    scope,
                    principal.principal_identity(),
                    &affinity,
                    &identities,
                    &execution,
                    &context,
                    &interruption,
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::conditional_definition::ConditionalDefinitionAdmission::new_for_test(
                        &world.application, admission.graph_work_for_test().mutation_product(),
                    ),
                );
                action(&mut decision)
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map(|completed| completed.into_parts().0)
    }).expect("the installed projection World remains available")
}
