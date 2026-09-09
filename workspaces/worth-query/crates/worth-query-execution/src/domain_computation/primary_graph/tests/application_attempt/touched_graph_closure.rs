use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::super::fixture::{
    AccountLabel, AccountStatus, MultiTouchOperation, TouchAccountOperation,
};

#[test]
fn incomplete_mandatory_decision_reads_cannot_form_an_effect_program() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let attempt = world
        .application
        .begin_application_read_attempt(admission)
        .unwrap();

    let Err(denial) = attempt.complete() else {
        panic!("an incomplete mandatory decision-read set must not complete");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::IncompleteDecisionReadSet
    );
}

#[test]
fn sealed_projection_completion_accepts_the_exact_empty_dependency_set() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |_, _| ())
        .unwrap()
        .into_parts();
    let attempt = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();

    attempt
        .complete_projected_dependencies()
        .expect("the projection sealed an exact empty dependency set");
}

#[test]
fn same_type_entity_outside_the_admitted_root_cannot_enter_an_unprojected_read_set() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let attempt = world
        .application
        .begin_application_read_attempt(admission)
        .unwrap();

    let Err(denial) = attempt.resolve_entity(AccountStatus::reference(), "unrelated".to_string())
    else {
        panic!("an unprojected read attempt must remain inside its admitted root");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope
    );
}

#[test]
fn only_the_exact_projection_occurrence_can_enter_its_read_set() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let other_account = resolved_account(&world, "unrelated", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();

    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let other_admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &other_account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, other_projection, _) = world
        .invariant
        .project_admitted_operation(&other_admission, |_, _| ())
        .unwrap()
        .into_parts();
    let mismatch = world
        .application
        .begin_projected_application_read_attempt(admission, other_projection)
        .err()
        .expect("another admitted scope's projection must not substitute");
    assert_eq!(
        mismatch.kind(),
        WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch
    );

    let first_equivalent = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let second_equivalent = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    assert_eq!(
        first_equivalent.operation_scope_binding(),
        second_equivalent.operation_scope_binding(),
        "equivalent retries intentionally retain one descriptive scope identity"
    );
    let (_, first_projection, _) = world
        .invariant
        .project_admitted_operation(&first_equivalent, |_, _| ())
        .unwrap()
        .into_parts();
    let equivalent_mismatch = world
        .application
        .begin_projected_application_read_attempt(second_equivalent, first_projection)
        .err()
        .expect("stable retry identity must not substitute occurrence authority");
    assert_eq!(
        equivalent_mismatch.kind(),
        WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch
    );
}

#[test]
fn projected_distinct_facts_cannot_exceed_the_installed_budget() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            reader
                .resolve_entity(AccountStatus::reference(), "unrelated".to_string())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let mut attempt = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let other = attempt
        .resolve_entity(AccountStatus::reference(), "unrelated".to_string())
        .unwrap();
    attempt
        .observe_field(&account, AccountStatus::reference())
        .unwrap();
    attempt
        .observe_field(&account, AccountLabel::reference())
        .unwrap();

    let Err(denial) = attempt.observe_field(&other, AccountStatus::reference()) else {
        panic!("a third distinct fact must not exceed the installed budget");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded
    );
}

#[test]
fn fact_budget_denial_precedes_freshness_provider_work() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let other = resolved_account(&world, "unrelated", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(&principal, &other, &operation, Default::default(), &request)
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            reader
                .resolve_entity(AccountStatus::reference(), "open".to_string())
                .unwrap();
        })
        .unwrap()
        .into_parts();

    let mutation = super::admitted_program(&world, &principal, &account, &request, "changed");
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(mutation, super::idempotency(21, 21)),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let newer_identity = resolved_account(&world, "changed", &request);

    let mut attempt = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    attempt
        .observe_field(&other, AccountStatus::reference())
        .unwrap();
    attempt
        .observe_field(&other, AccountLabel::reference())
        .unwrap();
    let denial = attempt
        .observe_field(&newer_identity, AccountStatus::reference())
        .expect_err("the third fact must deny before checking snapshot freshness");

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded
    );
}

#[test]
fn one_field_family_instance_cannot_satisfy_two_planned_entity_dependencies() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiTouchOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            let open = reader
                .resolve_entity(AccountStatus::reference(), "open".to_string())
                .unwrap();
            let unrelated = reader
                .resolve_entity(AccountStatus::reference(), "unrelated".to_string())
                .unwrap();
            reader
                .require_decision_field(&open, AccountStatus::reference())
                .unwrap();
            reader
                .require_decision_field(&unrelated, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let mut reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    reads
        .observe_field(&account, AccountStatus::reference())
        .unwrap();

    let denial = reads
        .complete()
        .err()
        .expect("one target-family instance cannot satisfy two exact planned facts");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::DecisionDependencyMismatch
    );
}
