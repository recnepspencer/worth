use bank_domain::model::{AccountId, BankPrincipalId, Money};
use bank_domain::schema::{
    Account, AccountingRevision, BankSchema, SendMoney, SendMoneyOperation, Status,
};
use bank_server::{
    BankApplicationAttemptDenialKind, BankCommitPreparationDenial, BankMutationCommitOutcome,
    BankOperationProposals, BankPrincipalSeed, BankWorldSeed,
};
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;

use super::fixture::{
    expect_send_proposal, funded_independent_sender_world, funded_personal_world, id, key,
};
use crate::support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

#[test]
fn expected_source_facts_reject_relevant_drift_during_commit_preparation() {
    let snapshot = funded_personal_world();
    let owner = DynamicIdentity::new("precondition-owner");
    let recipient = DynamicIdentity::new("precondition-recipient");
    let employee = DynamicIdentity::new("precondition-employee");
    let world = runtime(
        BankWorldSeed::new(snapshot.clone())
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 1),
                owner.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 2),
                recipient.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 3),
                employee.external(),
            )),
    );
    let request = request_scope();
    let actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&owner),
        &request,
    ))
    .unwrap();
    let source = snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap();
    let input = SendMoney {
        from: source,
        recipient: id(BankPrincipalId::new, 2),
        amount: Money::from_minor(250).unwrap(),
    };
    let first = prepare(
        &world.runtime,
        &actor,
        &request,
        &snapshot,
        source,
        &input,
        "precondition-first",
    );
    let BankMutationCommitOutcome::Committed(receipt) =
        world.runtime.commit_send_money(first).unwrap()
    else {
        panic!("the first matching attempt must commit");
    };
    assert_eq!(receipt.expected_version_count(), 1);
    assert_eq!(receipt.expected_fact_count(), 1);
    let stale = prepare(
        &world.runtime,
        &actor,
        &request,
        &snapshot,
        source,
        &input,
        "precondition-stale",
    );
    assert!(matches!(
        world.runtime.commit_send_money(stale),
        Err(BankCommitPreparationDenial::Application {
            kind: BankApplicationAttemptDenialKind::MutationPreconditionMismatch,
        })
    ));
}

#[test]
fn unrelated_drift_preserves_expected_source_facts() {
    let snapshot = funded_independent_sender_world();
    let identities = (1..=4)
        .map(|principal| DynamicIdentity::new(&format!("precondition-independent-{principal}")))
        .collect::<Vec<_>>();
    let mut seed = BankWorldSeed::new(snapshot.clone());
    for (ordinal, identity) in identities.iter().enumerate() {
        seed = seed.principal(BankPrincipalSeed::enabled(
            id(BankPrincipalId::new, (ordinal + 1) as u64),
            identity.external(),
        ));
    }
    let world = runtime(seed);
    let request = request_scope();
    let first_actor = authenticate(&world, &identities[0], &request);
    let second_actor = authenticate(&world, &identities[1], &request);
    let first_source = snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap();
    let second_source = snapshot
        .primary_account(id(BankPrincipalId::new, 2))
        .unwrap();
    let unrelated = prepare(
        &world.runtime,
        &second_actor,
        &request,
        &snapshot,
        second_source,
        &SendMoney {
            from: second_source,
            recipient: id(BankPrincipalId::new, 4),
            amount: Money::from_minor(250).unwrap(),
        },
        "precondition-independent-second",
    );

    assert!(matches!(
        world.runtime.commit_send_money(unrelated).unwrap(),
        BankMutationCommitOutcome::Committed(_)
    ));
    let first = prepare(
        &world.runtime,
        &first_actor,
        &request,
        &snapshot,
        first_source,
        &SendMoney {
            from: first_source,
            recipient: id(BankPrincipalId::new, 3),
            amount: Money::from_minor(250).unwrap(),
        },
        "precondition-independent-first",
    );
    let BankMutationCommitOutcome::Committed(receipt) =
        world.runtime.commit_send_money(first).unwrap()
    else {
        panic!("unrelated drift must preserve the expected source facts");
    };
    assert_eq!(receipt.expected_version_count(), 1);
    assert_eq!(receipt.expected_fact_count(), 1);
}

fn prepare(
    runtime: &bank_server::BankIdentityRuntime,
    actor: &bank_server::BankAuthenticatedPrincipal,
    request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    snapshot: &bank_domain::proposals::BankSnapshot,
    source: AccountId,
    input: &SendMoney,
    idempotency_key: &str,
) -> bank_server::BankAuthorizedProposal<SendMoneyOperation, SendMoney, Account, AccountId> {
    let account = snapshot.account(source).unwrap();
    use worth_query_host::facade::declaration::application_schema::ApplicationEncodedScalarValue;

    let preconditions =
        TypedMutationPreconditions::<BankSchema, SendMoneyOperation, Account>::new()
            .expect_version(
                AccountingRevision::reference(),
                ApplicationEncodedScalarValue::<bank_domain::schema::AccountJournalRevisionBinding>::try_new(
                    snapshot.account_journal_revision(source).unwrap(),
                )
                .unwrap(),
            )
            .expect_fact(
                Status::reference(),
                ApplicationEncodedScalarValue::<bank_domain::schema::AccountStatusBinding>::try_new(
                    account.status(),
                )
                .unwrap(),
            );
    let admission = runtime
        .authorize_send_money(actor, source, preconditions, request)
        .unwrap();
    expect_send_proposal(
        BankOperationProposals::prepare_send_money(
            runtime,
            admission,
            &key(idempotency_key),
            input,
        )
        .unwrap(),
    )
}

fn authenticate(
    world: &crate::support::TestIdentityWorld,
    identity: &DynamicIdentity,
    request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> bank_server::BankAuthenticatedPrincipal {
    block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(identity),
        request,
    ))
    .unwrap()
}
