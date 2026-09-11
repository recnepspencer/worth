use std::time::Duration;

use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, StringApplicationValueBinding, TypedMutationPreconditions,
};

use super::super::fixture::{
    installed_authorization_world, live_scope, AccountStatus, TouchAccountOperation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn installed_precondition_entry_and_byte_budgets_fail_closed() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let principal = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &world.binding,
            external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
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
            AccountStatus::reference(),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "open".to_owned(),
            )
            .expect("fixture account status must encode"),
        )
        .expect_fact(
            AccountStatus::reference(),
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                "open".to_owned(),
            )
            .expect("fixture account status must encode"),
        );
    assert_precondition_rejected(
        world.selected_product().authorize_operation(
            &principal,
            &account,
            &operation,
            duplicate_target,
            &request,
        ),
        "duplicate targets must not inflate work beyond the installed entry ceiling",
    );

    let oversized_value = "x".repeat(budget.maximum_encoded_bytes());
    let oversized = TypedMutationPreconditions::new().expect_fact(
        AccountStatus::reference(),
        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(oversized_value)
            .expect("fixture account status must encode"),
    );
    assert_precondition_rejected(
        world
            .selected_product()
            .authorize_operation(&principal, &account, &operation, oversized, &request),
        "canonical material beyond the installed byte ceiling must be denied",
    );
}

fn assert_precondition_rejected<T>(
    outcome: Result<T, WorthQueryOperationAuthorizationDenial>,
    message: &str,
) {
    let denial = outcome.err().unwrap_or_else(|| panic!("{message}"));
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::MutationPreconditionRejected,
        "{message}"
    );
}
