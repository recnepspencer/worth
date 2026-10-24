//! The same owner pins the initiating principal and rechecks fresh command authority.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryCapabilityCommandAdvance as Advance, WorthQueryCapabilityCommandDenial as Denial,
};

#[test]
fn retained_delegation_cannot_retarget_and_rechecks_command_grant() {
    let fixture = delegation_world("owning-delegation-principal-and-grant");
    let original = fixture.authenticate();
    let other = fixture.authenticate_approver();
    let scope = request_scope();
    let command = fixture
        .runtime
        .retain_estate_capability_delegation(
            &original,
            delegated_action(DelegationLimit::generations(1)),
            idempotency(141),
            &scope,
        )
        .unwrap_or_else(|_| panic!("capture original funded descriptor"));
    let command = match command.advance(&fixture.runtime.request(&other, &scope)) {
        Advance::Refused { command, denial: Denial::Native(
            worth_query_host::facade::application_entry::WorthQueryApplicationCapabilityDelegationDenial::PrincipalMismatch,
        ) } => command,
        _ => panic!("different authorized principal cannot retarget original command"),
    };
    assert!(capability_if_present(
        &governance_readback(&fixture, &fixture.authenticate_executor()),
        CHILD
    )
    .is_none());
    let revoked = crate::estate_progression::delegation::command_courts::complete_revocation(
        &fixture.runtime,
        &original,
        EstateAction::RevokeCapability {
            estate: ESTATE,
            grant: crate::estate_capability_admission::fixture::COMMAND_GRANT,
        },
        &idempotency(142),
        &scope,
    )
    .expect("real revocation removes command access after capture");
    assert_committed(revoked);
    let command = match command.advance(&fixture.runtime.request(&original, &scope)) {
        Advance::Refused { command, denial: Denial::Native(
            worth_query_host::facade::application_entry::WorthQueryApplicationCapabilityDelegationDenial::Authorization(_),
        ) } => command,
        _ => panic!("original input and pin never substitute for fresh command authority"),
    };
    assert!(capability_if_present(
        &governance_readback(&fixture, &fixture.authenticate_executor()),
        CHILD
    )
    .is_none());
    assert!(
        command
            .close(&fixture.runtime.request(&original, &scope))
            .is_ok(),
        "unused issuer owner can close without reacquiring revoked command access"
    );
}
