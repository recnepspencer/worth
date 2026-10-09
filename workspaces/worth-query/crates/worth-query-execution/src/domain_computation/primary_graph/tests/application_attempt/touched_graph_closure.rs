use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

use super::super::fixture::{AccountStatus, MultiTouchOperation, TouchAccountOperation};

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

    let Err(denial) =
        attempt.complete(crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
    else {
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
        .project_admitted_operation(&admission, |_, _| (), AllocationPolicy::SystemAllocation)
        .unwrap()
        .into_parts();
    let attempt = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();

    attempt
        .complete_projected_dependencies(
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
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
        .project_admitted_operation(
            &other_admission,
            |_, _| (),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let mismatch = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            other_projection,
            AllocationPolicy::SystemAllocation,
        )
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
        .project_admitted_operation(
            &first_equivalent,
            |_, _| (),
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let equivalent_mismatch = world
        .application
        .begin_projected_application_read_attempt(
            second_equivalent,
            first_projection,
            AllocationPolicy::SystemAllocation,
        )
        .err()
        .expect("stable retry identity must not substitute occurrence authority");
    assert_eq!(
        equivalent_mismatch.kind(),
        WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch
    );
}

#[path = "touched_graph_closure/field_family_occurrence.rs"]
mod field_family_occurrence;

#[path = "touched_graph_closure/stale_identity.rs"]
mod stale_identity;
