//! The same owner pins the initiating principal and recovers actual ambiguous publication.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryCapabilityCommandAdvance as Advance,
    WorthQueryCapabilityCommandCompletion as Completion,
    WorthQueryCapabilityCommandDenial as Denial,
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

#[test]
fn retained_delegation_recovers_original_publication_and_refuses_cancelled_cleanup() {
    let fixture = delegation_world("owning-delegation-native-ambiguous-close");
    let original = fixture.authenticate();
    let scope = request_scope();
    let mut command = fixture
        .runtime
        .retain_estate_capability_delegation(
            &original,
            delegated_action(DelegationLimit::generations(1)),
            idempotency(143),
            &scope,
        )
        .unwrap_or_else(|_| panic!("capture original funded command"));
    fixture.runtime.fail_next_durable_append_for_test();
    command = match command.advance(&fixture.runtime.request(&original, &scope)) {
        Advance::Progress(command) => command,
        _ => panic!("native durable append failure retains original ambiguous command"),
    };
    assert!(
        capability_if_present(
            &governance_readback(&fixture, &fixture.authenticate_executor()),
            CHILD
        )
        .is_none(),
        "partial source publication is not current child publication"
    );
    let completed = loop {
        match command.advance(&fixture.runtime.request(&original, &request_scope())) {
            Advance::Progress(next) => command = next,
            Advance::Completed(completed) => break completed,
            _ => panic!("same original owner performs native recovery and keyed settlement"),
        }
    };
    assert!(
        matches!(
            completed.evidence(),
            Completion::Recovered {
                publication: Some(_),
                ..
            }
        ),
        "recovery retains genuine native performed publication evidence"
    );
    let visible = governance_readback(&fixture, &fixture.authenticate_executor());
    assert_eq!(
        visible.rows()[0]
            .capabilities()
            .iter()
            .filter(|grant| grant.id() == CHILD)
            .count(),
        1,
        "one original child, not a repeated command effect"
    );
    let cancelled = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    cancelled.cancel();
    let cancelled_scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancelled.token(),
        );
    let (completed, denial) = completed
        .close(&fixture.runtime.request(&original, &cancelled_scope))
        .err()
        .expect("native cleanup refuses actual cancelled admission while preserving evidence");
    assert!(matches!(denial, Denial::Execution(_)));
    assert!(matches!(
        completed.evidence(),
        Completion::Recovered {
            publication: Some(_),
            ..
        }
    ));
    let closed = completed
        .close(&fixture.runtime.request(&original, &request_scope()))
        .unwrap_or_else(|_| panic!("fresh issuer request lawfully releases original collection"));
    assert!(closed.retired_component_count() > 0,
        "recovered native record custody is actually retired; empty cleanup cannot satisfy this court");
}
