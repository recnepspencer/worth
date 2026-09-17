use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::*;
use crate::estate_capability_admission::fixture::{delegation_world_with_parent_spec, GrantSpec};

#[test]
fn parent_expiry_after_activation_materialization_denies_final_commit() {
    let expiry = epoch_seconds() + 20;
    let mut parent = GrantSpec::governance_view();
    parent.not_after = expiry;
    let fixture = delegation_world_with_parent_spec("delegation-provider-parent-expiry", parent);
    let specialist = fixture.authenticate();
    let mut action = delegated_action();
    let EstateAction::DelegateCapability { child, .. } = &mut action else {
        unreachable!("the fixture action is delegation")
    };
    child.scope.validity = CapabilityValidity::new(
        EstateMoment::from_epoch_seconds(0),
        EstateMoment::from_epoch_seconds(expiry),
    )
    .unwrap();
    let command = delegation_command(action).unwrap();
    let admission = fixture
        .runtime
        .admit_delegation(&specialist, action, command.child, &request_scope())
        .unwrap();
    let program = fixture
        .runtime
        .materialize_delegation(admission, command.child)
        .expect("the activation program must materialize before parent expiry");

    while epoch_seconds() <= expiry {
        std::thread::sleep(Duration::from_millis(10));
    }
    let outcome = fixture
        .runtime
        .application_program()
        .admit_program_operation::<DelegateEstateCapabilityOperation>()
        .unwrap()
        .compare_and_commit_capability_delegation(program, query_idempotency(127));
    assert_provider_currentness_denial(outcome);
    assert_child_absent(&fixture);
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test time follows the Unix epoch")
        .as_secs()
}
