use super::*;

/// A prepared delegation decided on its parent's authority and on its child
/// id naming no grant. An unrelated revocation moves neither fact. A commit
/// of that child id ends the second for every other request.
#[test]
fn unrelated_revocation_preserves_the_prepared_delegation_and_a_taken_child_id_stales() {
    let fixture = delegation_world("delegation-provider-unrelated-currentness");
    let specialist = fixture.authenticate();
    let action = delegated_action();
    let command = delegation_command(action).unwrap();
    let prepare = || {
        let admission = fixture
            .runtime
            .admit_delegation(&specialist, action, command.child, &request_scope())
            .unwrap();
        fixture
            .runtime
            .materialize_delegation(admission, command.child)
            .expect("the exact activation program must retain only relevant support")
    };
    let program = prepare();
    let retry = prepare();
    let losing = prepare();

    let revoked = fixture
        .runtime
        .revoke_estate_capability_with_key(
            &specialist,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: UNRELATED_GOVERNANCE_GRANT,
            },
            &idempotency(139),
            &request_scope(),
        )
        .expect("an unrelated authority should revoke independently");
    assert!(matches!(revoked, BankMutationCommitOutcome::Committed(_)));
    let commit = |program, seed| {
        fixture
            .runtime
            .application_program()
            .admit_program_operation::<DelegateEstateCapabilityOperation>()
            .unwrap()
            .compare_and_commit_capability_delegation(program, query_idempotency(seed))
    };
    // The revocation changed no fact the prepared delegation decided on, so
    // the program commits on the current product.
    let outcome = commit(program, 141);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "an unrelated revocation preserves the prepared delegation: {outcome:?}"
    );
    assert_eq!(grants_named(&fixture, &specialist, CHILD), 1);
    let replayed = commit(retry, 141);
    assert!(
        matches!(
            replayed,
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "the same request replays its commit: {replayed:?}"
    );

    // Another request prepared for the same child id decided on that id
    // naming no grant. The commit ended that fact.
    let lost = commit(losing, 143);
    let WorthQueryApplicationCommitOutcome::Stale(stale) = lost else {
        panic!("a taken child id stales the other prepared delegation: {lost:?}");
    };
    assert!(stale.stale_fact_count() > 0);
    assert_eq!(grants_named(&fixture, &specialist, CHILD), 1);

    // A request admitted after the commit reads the grant the id names and
    // is denied by the domain rule.
    let late = fixture.runtime.delegate_estate_capability_with_key(
        &specialist,
        action,
        &idempotency(145),
        &request_scope(),
    );
    assert!(
        matches!(
            late,
            Err(BankEstateProgressionDenial::CapabilityDelegationProjection(
                BankCapabilityDelegationProjectionDenial::ChildGrantExists
            ))
        ),
        "a taken child id denies a later delegation: {late:?}"
    );
    assert_eq!(grants_named(&fixture, &specialist, CHILD), 1);
}

fn grants_named(
    fixture: &crate::estate_capability_admission::fixture::CapabilityFixture,
    principal: &BankAuthenticatedPrincipal,
    grant: CapabilityGrantId,
) -> usize {
    let result = fixture
        .runtime
        .query(crate::queries::estate_governance_context(ESTATE))
        .as_principal(principal)
        .controls(BankReadControls::current(request_scope(), 1, 20_000).unwrap())
        .execute()
        .expect("governance authority must read authoritative delegation state");
    result.rows()[0]
        .capabilities()
        .iter()
        .filter(|capability| capability.id() == grant)
        .count()
}
