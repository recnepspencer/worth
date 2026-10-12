use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::super::fixture::{AccountLabel, MultiTouchOperation};

#[test]
fn projection_capability_does_not_widen_the_installed_decision_manifest() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
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
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                )
                .authorize_operation(
                    &host_custody.phase_for_application(&world.application),
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
