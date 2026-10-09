//! A later-resolved identity cannot enter an earlier read basis or contact its value provider.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::take_native_field_contacts;

#[test]
fn stale_identity_is_refused_before_counted_native_field_provider_work() {
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
        .project_admitted_operation(
            &admission,
            |reader, _| {
                reader
                    .resolve_entity(AccountStatus::reference(), "open".to_owned())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let mutation =
        super::super::admitted_program(&world, &principal, &account, &request, "changed");
    assert!(matches!(
        world.application.compare_and_commit_application(
            mutation,
            super::super::idempotency(21, 21),
            AllocationPolicy::SystemAllocation,
        ),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let newer = resolved_account(&world, "changed", &request);
    let mut attempt = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap();
    take_native_field_contacts();
    attempt
        .observe_field(&other, AccountStatus::reference())
        .unwrap();
    assert_eq!(
        take_native_field_contacts(),
        1,
        "a current identity really contacts the native value provider"
    );
    let denial = attempt
        .observe_field(&newer, AccountStatus::reference())
        .unwrap_err();
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
