use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::MultiTouchOperation;

#[test]
fn projection_capability_does_not_widen_the_installed_decision_manifest() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application_for_test(&world.application))
                    .test_host_request_for_test(),
                &world,
                "open",
                &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(MultiTouchOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(
                    &(host_custody.phase_for_application_for_test(&world.application))
                        .test_host_request_for_test(),
                )
                .authorize_operation(
                    &host_custody.phase_for_application_for_test(&world.application),
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let mut attempt = world
                .application
                .begin_application_read_attempt(execution, admission)
                .unwrap();

            let Err(denial) = attempt.observe_field(execution, &account, AccountLabel::reference())
            else {
                panic!(
                    "a compile-capable projection field must still need installed read admission"
                );
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead
            );
        })
        .expect("the fixture block owns its installed host call")
}
