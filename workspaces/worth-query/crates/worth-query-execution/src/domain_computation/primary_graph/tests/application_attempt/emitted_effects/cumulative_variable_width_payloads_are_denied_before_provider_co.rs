//! cumulative variable width payloads are denied before provider commit.
use super::*;

#[test]
fn cumulative_variable_width_payloads_are_denied_before_provider_commit() {
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
                    |reader, projected| {
                        reader
                            .require_decision_field(projected, AccountStatus::reference())
                            .unwrap();
                    },
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
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
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");
    let first = String::with_capacity(140_000);
    let second = String::with_capacity(140_000);

    effects
        .emit(AccountActivityEffect::reference(), first)
        .expect("one variable-width payload fits the installed byte envelope");
    let denial = effects
        .emit(AccountActivityEffect::reference(), second)
        .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::RetainedEffectBytesExceeded
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0,
        "payload admission must fail before provider mutation or publication"
    );
}
