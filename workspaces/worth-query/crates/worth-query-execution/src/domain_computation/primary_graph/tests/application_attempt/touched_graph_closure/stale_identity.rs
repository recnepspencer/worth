//! A later-resolved identity cannot enter an earlier read basis or contact its value provider.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::take_native_field_contacts;

#[test]
fn stale_identity_is_refused_before_counted_native_field_provider_work() {
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
    let other = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
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
                    &other,
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
                    |reader, _| {
                        reader
                            .resolve_entity(AccountStatus::reference(), "open".to_owned())
                            .unwrap();
                    },
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mutation = world
        .application
        .with_host_advancement(|phase| {
            super::super::admitted_program(
                &phase, &world, &principal, &account, &request, "changed",
            )
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        world.application.compare_and_commit_application(
            mutation,
            super::super::idempotency(21, 21),
            AllocationPolicy::SystemAllocation,
        ),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let newer = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "changed", &request)
        })
        .expect("the fixture read owns its installed host call");
    let mut attempt = world
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
    take_native_field_contacts();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            attempt
                .observe_field(
                    &host_custody.borrowed_request_for_test(),
                    &other,
                    AccountStatus::reference(),
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        take_native_field_contacts(),
        1,
        "a current identity really contacts the native value provider"
    );
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            attempt
                .observe_field(
                    &host_custody.borrowed_request_for_test(),
                    &newer,
                    AccountStatus::reference(),
                )
                .unwrap_err()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::StaleEntityIdentity
    );
    assert_eq!(
        take_native_field_contacts(),
        0,
        "stale identity refusal precedes native field-provider work"
    );
}
