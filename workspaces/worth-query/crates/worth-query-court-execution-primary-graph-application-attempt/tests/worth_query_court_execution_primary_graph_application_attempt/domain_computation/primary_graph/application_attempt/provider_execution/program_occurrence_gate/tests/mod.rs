//! Native occurrence-owner evidence; producer entry propagation is tested separately.

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::program_occurrence_gate::require_occurrence_acts_through;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_execution::program_occurrence_gate::resolve_occurrence_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::installed_program_support;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::rostered_program_revision;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_activation::seed_program_activation;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;

#[test]
fn active_program_names_an_operation_missing_from_its_declared_routes() {
    let mut world = installed_authorization_world(true);
    (*world.application.program_support_mut_for_test()) = Some(installed_program_support(
        &(*world.application.installed_schema_for_test()),
    ));

    let request = live_scope();
    world.application.with_host_advancement(|phase| {
let host_custody = phase.test_host_request_for_test();
let borrowed_execution = host_custody.borrowed_request_for_test();
let execution = &borrowed_execution;
seed_program_activation(execution, &world, &rostered_program_revision());
let principal = authenticated_principal(
            &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            &world,
            &request,
        );
        let account = resolved_account(
            &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            &world,
            "open",
            &request,
        );
        let predecessor = world
            .selected_product(
                &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
            )
            .product()
            .selected_commit()
            .clone();
        let admission = admitted_operation(
            &host_custody.phase_for_application_for_test(&world.application),
            &world,
            &principal,
            &account,
            &request,
        );
        let (_, projection, _) = world
            .invariant
            .project_admitted_operation(
                &host_custody.phase_for_application_for_test(&world.application),
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
        let program = world
            .application
            .begin_projected_application_read_attempt(
                &host_custody.phase_for_application_for_test(&world.application),
                admission,
                projection,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .complete_projected_dependencies(
                &host_custody.phase_for_application_for_test(&world.application),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .begin_effect_program()
            .finish()
            .unwrap();
        let support = (*world.application.program_support_for_test()).as_ref().unwrap();
        let occurrence = resolve_occurrence_program(execution, support, &program).unwrap();
        let denial =
            require_occurrence_acts_through::<TouchAccountOperation>(&occurrence).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
        );
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::ProposalBinding
        );
        let expected = format!(
        "operation {} is not declared by active program worth.query.test.program-required-program.v1 revision {}",
        std::any::type_name::<TouchAccountOperation>(),
        rostered_program_revision(),
    );
        assert_eq!(denial.detail(), Some(expected.as_str()));
        assert_eq!(
            world
                .selected_product(
                    &(host_custody.phase_for_application_for_test(&world.application)).test_host_request_for_test(),
                )
                .product()
                .selected_commit(),
            &predecessor
        );

}).expect("the fixture block owns its installed host call")
}
