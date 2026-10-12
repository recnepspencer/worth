use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

use super::super::fixture::{AccountStatus, MultiTouchOperation, TouchAccountOperation};

#[test]
fn incomplete_mandatory_decision_reads_cannot_form_an_effect_program() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let attempt = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .begin_application_read_attempt(
                    &host_custody.borrowed_request_for_test(),
                    admission,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |_, _| (),
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let attempt = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            attempt
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .expect("the projection sealed an exact empty dependency set")
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn same_type_entity_outside_the_admitted_root_cannot_enter_an_unprojected_read_set() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let attempt = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .begin_application_read_attempt(
                    &host_custody.borrowed_request_for_test(),
                    admission,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            attempt.resolve_entity(&phase, AccountStatus::reference(), "unrelated".to_string())
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("an unprojected read attempt must remain inside its admitted root");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope
    );
}

#[path = "touched_graph_closure/only_the_exact_projection_occurrence_can_enter_its_read_set.rs"]
mod only_the_exact_projection_occurrence_can_enter_its_read_set;

#[path = "touched_graph_closure/field_family_occurrence.rs"]
mod field_family_occurrence;

#[path = "touched_graph_closure/stale_identity.rs"]
mod stale_identity;
