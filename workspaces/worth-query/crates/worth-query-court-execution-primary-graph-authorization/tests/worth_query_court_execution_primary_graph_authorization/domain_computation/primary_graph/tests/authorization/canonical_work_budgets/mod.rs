use std::time::Duration;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn installed_precondition_entry_and_byte_budgets_fail_closed() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .select_product_branch_for_test(
                    &host_custody.borrowed_request_for_test(),
                    world
                        .application
                        .product_runtime()
                        .default_branch_for_test(),
                )
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let budget = operation
        .contracts()
        .precondition_canonical_work_budget()
        .expect("the installed precondition family owns a nonzero canonical budget");

    assert_eq!(budget.maximum_entry_count(), 6);
    assert_eq!(budget.maximum_encoded_bytes(), 256 * 1_024);

    let duplicate_target = TypedMutationPreconditions::new()
        .expect_fact(
            AccountStatus::at("AccountStatus"),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "open".to_owned(),
            )
            .expect("fixture account status must encode"),
        )
        .expect("record field contract")
        .expect_fact(
            AccountStatus::at("AccountStatus"),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "open".to_owned(),
            )
            .expect("fixture account status must encode"),
        )
        .expect("record field contract");
    world
        .application
        .with_host_advancement(|phase| {
            assert_precondition_rejected(
                world
                    .selected_product(&(phase).test_host_request_for_test())
                    .authorize_operation(
                        &phase,
                        &principal,
                        &account,
                        &operation,
                        duplicate_target,
                        &request,
                    ),
                "duplicate targets must not inflate work beyond the installed entry ceiling",
            )
        })
        .expect("the fixture read owns its installed host call");

    let oversized_value = "x".repeat(budget.maximum_encoded_bytes());
    let oversized = TypedMutationPreconditions::new()
        .expect_fact(
            AccountStatus::at("AccountStatus"),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                oversized_value,
            )
            .expect("fixture account status must encode"),
        )
        .expect("record field contract");
    world
        .application
        .with_host_advancement(|phase| {
            assert_precondition_rejected(
                world
                    .selected_product(&(phase).test_host_request_for_test())
                    .authorize_operation(
                        &phase, &principal, &account, &operation, oversized, &request,
                    ),
                "canonical material beyond the installed byte ceiling must be denied",
            )
        })
        .expect("the fixture read owns its installed host call");
}

pub fn assert_precondition_rejected<T>(
    outcome: Result<T, WorthQueryOperationAuthorizationDenial>,
    message: &str,
) {
    let denial = outcome.err().unwrap_or_else(|| panic!("{message}"));
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::MutationPreconditionRejected
        ),
        "{message}"
    );
}
