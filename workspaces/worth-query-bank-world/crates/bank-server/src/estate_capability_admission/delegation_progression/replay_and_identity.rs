use super::*;
use crate::estate_capability_admission::fixture::{
    delegation_world_with_parent_spec_at, AuthorizationTimeController, GrantSpec,
};

#[test]
fn two_principals_can_delegate_with_the_same_client_key() {
    let fixture = delegation_world("capability-delegation-principal-key-scope");
    let specialist = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let key = idempotency(89);

    assert_committed(
        fixture
            .runtime
            .delegate_estate_capability_with_key(
                &specialist,
                delegated_action(DelegationLimit::generations(1)),
                &key,
                &request_scope(),
            )
            .expect("the specialist should activate the first child"),
    );
    assert_committed(
        fixture
            .runtime
            .delegate_estate_capability_with_key(
                &approver,
                delegated_action_from(CHILD, GRANDCHILD, REVIEWER, DelegationLimit::generations(0)),
                &key,
                &request_scope(),
            )
            .expect("a second authorized principal should use the same client key independently"),
    );
    let visible = governance_readback(&fixture, &fixture.authenticate_reviewer());
    assert!(capability_if_present(&visible, GRANDCHILD).is_some());
}

#[test]
fn two_principals_can_revoke_with_the_same_client_key() {
    let fixture = delegation_world("capability-revocation-principal-key-scope");
    let specialist = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let key = idempotency(90);

    assert_committed(
        fixture
            .runtime
            .revoke_estate_capability_with_key(
                &specialist,
                EstateAction::RevokeCapability {
                    estate: ESTATE,
                    grant: GRANT,
                },
                &key,
                &request_scope(),
            )
            .expect("the specialist should revoke the first grant"),
    );
    assert_committed(
        fixture
            .runtime
            .revoke_estate_capability_with_key(
                &approver,
                EstateAction::RevokeCapability {
                    estate: ESTATE,
                    grant: UNRELATED_GOVERNANCE_GRANT,
                },
                &key,
                &request_scope(),
            )
            .expect("a second authorized principal should use the same client key independently"),
    );
}

#[test]
fn exact_delegation_replays_after_parent_expiry_but_fresh_child_is_denied() {
    let time = AuthorizationTimeController::at_epoch_seconds(100);
    let fixture = delegation_world_with_parent_spec_at(
        "capability-delegation-expired-parent-replay",
        GrantSpec {
            not_after: 200,
            ..GrantSpec::governance_view()
        },
        time.clone(),
    );
    let principal = fixture.authenticate();
    let mut action = delegated_action(DelegationLimit::generations(1));
    let EstateAction::DelegateCapability { child, .. } = &mut action else {
        unreachable!();
    };
    child.scope.validity = CapabilityValidity::new(
        EstateMoment::from_epoch_seconds(0),
        EstateMoment::from_epoch_seconds(200),
    )
    .unwrap();
    let key = idempotency(91);

    assert_committed(delegate(&fixture, &principal, action, key.clone()).unwrap());
    time.advance_to_epoch_seconds(201);
    let replay = delegate(&fixture, &principal, action, key.clone())
        .expect("an exact prior commit must replay after its parent expires");
    assert!(matches!(
        replay,
        BankMutationCommitOutcome::AlreadyCommitted(_)
    ));

    let EstateAction::DelegateCapability { child, .. } = &mut action else {
        unreachable!();
    };
    child.id = DRIFTED_CHILD;
    let drift_denial = delegate(&fixture, &principal, action, key)
        .expect_err("a changed child cannot reuse the expired parent's prior receipt");
    assert!(matches!(
        drift_denial,
        BankEstateProgressionDenial::Authorization(_)
    ));
    let denial = delegate(&fixture, &principal, action, idempotency(92))
        .expect_err("an expired parent must not activate a new child");
    assert!(matches!(
        denial,
        BankEstateProgressionDenial::Authorization(_)
    ));
}
