use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::super::fixture::{AccountLabel, MultiTouchOperation};

#[test]
fn projection_capability_does_not_widen_the_installed_decision_manifest() {
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
    let mut attempt = world
        .application
        .begin_application_read_attempt(admission)
        .unwrap();

    let Err(denial) = attempt.observe_field(&account, AccountLabel::reference()) else {
        panic!("a compile-capable projection field must still need installed read admission");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead
    );
}
