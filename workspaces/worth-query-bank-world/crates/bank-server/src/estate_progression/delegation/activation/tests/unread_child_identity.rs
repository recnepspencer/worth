//! A delegation whose application never reads its child id still decides on
//! that id naming no grant: Query appends the absence of every unique value
//! the activation creates, so a second grant with the same id never commits.

use super::prepared_currentness::grants_named;
use super::*;

#[test]
fn a_delegation_that_does_not_read_its_child_id_stales_then_is_denied() {
    let fixture = delegation_world("delegation-unread-child-identity");
    let specialist = fixture.authenticate();
    let action = delegated_action();
    let first = materialize_unread(&fixture.runtime, &specialist, action);
    let second = materialize_unread(&fixture.runtime, &specialist, action);

    let committed = commit(&fixture, first, 171);
    assert!(
        matches!(committed, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the first delegation commits: {committed:?}"
    );
    let lost = commit(&fixture, second, 173);
    let WorthQueryApplicationCommitOutcome::Stale(stale) = lost else {
        panic!("the appended absence stales the second delegation: {lost:?}");
    };
    assert!(stale.stale_fact_count() > 0);
    assert_eq!(grants_named(&fixture, &specialist, CHILD), 1);

    // A request prepared after the commit observes the grant and is denied
    // by the unique law, with its own kind.
    let late = commit(
        &fixture,
        materialize_unread(&fixture.runtime, &specialist, action),
        175,
    );
    assert_unique_value_taken(late);
    assert_eq!(grants_named(&fixture, &specialist, CHILD), 1);

    // The committed request's retry resolves its record before the law runs.
    let retry = commit(
        &fixture,
        materialize_unread(&fixture.runtime, &specialist, action),
        171,
    );
    assert!(
        matches!(
            retry,
            WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
        ),
        "a retry of the committed delegation replays: {retry:?}"
    );
}

/// A grant id is never reused: once a grant's status changes, its id still
/// names it, so delegating that id again is denied.
#[test]
fn a_revoked_child_id_cannot_be_delegated_again() {
    let fixture = delegation_world("delegation-unread-revoked-child");
    let specialist = fixture.authenticate();
    let action = delegated_action();
    let delegated = fixture
        .runtime
        .delegate_estate_capability_with_key(
            &specialist,
            action,
            &idempotency(177),
            &request_scope(),
        )
        .expect("the child delegates");
    assert!(matches!(delegated, BankMutationCommitOutcome::Committed(_)));
    let revoked = fixture
        .runtime
        .revoke_estate_capability_with_key(
            &specialist,
            EstateAction::RevokeCapability {
                estate: ESTATE,
                grant: CHILD,
            },
            &idempotency(179),
            &request_scope(),
        )
        .expect("the child's status changes under the unique id");
    assert!(matches!(revoked, BankMutationCommitOutcome::Committed(_)));

    let again = commit(
        &fixture,
        materialize_unread(&fixture.runtime, &specialist, action),
        181,
    );
    assert_unique_value_taken(again);
    assert_eq!(grants_named(&fixture, &specialist, CHILD), 1);
}

/// The bank reads its child id itself, so its own rule denies a taken id
/// before the unique law is reached.
#[test]
fn the_bank_s_own_child_rule_is_seen_before_the_unique_law() {
    let fixture = delegation_world("delegation-unread-denial-order");
    let specialist = fixture.authenticate();
    let action = delegated_action();
    fixture
        .runtime
        .delegate_estate_capability_with_key(
            &specialist,
            action,
            &idempotency(183),
            &request_scope(),
        )
        .expect("the child delegates");
    let late = fixture.runtime.delegate_estate_capability_with_key(
        &specialist,
        action,
        &idempotency(185),
        &request_scope(),
    );
    assert!(
        matches!(
            late,
            Err(BankEstateProgressionDenial::CapabilityDelegationProjection(
                BankCapabilityDelegationProjectionDenial::ChildGrantExists
            ))
        ),
        "the bank's projection denies first: {late:?}"
    );
}

fn assert_unique_value_taken(outcome: WorthQueryApplicationCommitOutcome) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("a taken child id denies the delegation: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::UniqueValueTaken
    );
}

fn commit(
    fixture: &crate::estate_capability_admission::fixture::CapabilityFixture,
    program: DelegationProgram,
    seed: u8,
) -> WorthQueryApplicationCommitOutcome {
    fixture
        .runtime
        .application_program()
        .admit_program_operation::<DelegateEstateCapabilityOperation>()
        .unwrap()
        .compare_and_commit_capability_delegation(program, query_idempotency(seed))
}

/// Materializes the delegation from a projection that checks the scope's
/// relations and never selects the child id.
fn materialize_unread(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    action: EstateAction,
) -> DelegationProgram {
    let child = delegation_command(action).unwrap().child;
    let admission = runtime
        .admit_delegation(principal, action, child, &request_scope())
        .expect("the delegation admits");
    let projected = runtime
        .invariant_projection()
        .project_admitted_operation(&admission, |reader, estate| {
            let branch =
                reader.resolve_entity(BranchIdentityField::reference(), child.scope.branch)?;
            let institution = reader.resolve_entity(
                InstitutionIdentityField::reference(),
                child.scope.institution,
            )?;
            reader.require_decision_relation(EstateBranch::reference(), estate, &branch)?;
            reader.require_decision_relation(
                BranchInstitution::reference(),
                &branch,
                &institution,
            )?;
            Ok::<_, BankCapabilityDelegationProjectionDenial>(())
        })
        .expect("the scope projects");
    let (result, projection, _) = projected.into_parts();
    result.expect("the scope's relations hold");
    runtime
        .application_runtime()
        .begin_projected_application_read_attempt(admission, projection)
        .expect("the read attempt begins")
        .complete_projected_dependencies()
        .expect("the dependencies complete")
        .materialize_capability_delegation_program()
        .expect("Query materializes the activation with its child id's absence")
}
