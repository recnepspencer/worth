use bank_domain::model::{BankPrincipalId, Money};
use bank_domain::proposals::{BankProposalDenial, BankProposedEffect};
use bank_domain::schema::SendMoney;
use bank_server::{
    BankMutationCommitOutcome, BankMutationProjectionWork, BankOperationProposalError,
    BankOperationProposals, BankPrincipalSeed, BankWorldSeed,
};

use super::fixture::{
    expect_send_proposal, funded_independent_sender_world, funded_personal_world,
    funded_personal_world_with_unrelated_journals, id, key, unrelated_principal,
};
use crate::support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

#[test]
fn concurrent_same_basis_transfers_cannot_overspend_one_account() {
    let snapshot = funded_personal_world();
    let owner = DynamicIdentity::new("concurrent-owner");
    let recipient = DynamicIdentity::new("concurrent-recipient");
    let employee = DynamicIdentity::new("concurrent-employee");
    let world = std::sync::Arc::new(runtime(
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
    ));
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
        amount: Money::from_minor(6_000).unwrap(),
    };
    let first_admission = world
        .runtime
        .authorize_send_money(&actor, source, Default::default(), &request)
        .unwrap();
    let second_admission = world
        .runtime
        .authorize_send_money(&actor, source, Default::default(), &request)
        .unwrap();
    let first = expect_send_proposal(
        BankOperationProposals::prepare_send_money(
            &world.runtime,
            first_admission,
            &key("concurrent-first"),
            &input,
        )
        .unwrap(),
    );
    let second = expect_send_proposal(
        BankOperationProposals::prepare_send_money(
            &world.runtime,
            second_admission,
            &key("concurrent-second"),
            &input,
        )
        .unwrap(),
    );

    let first_runtime = std::sync::Arc::clone(&world);
    let first = std::thread::spawn(move || first_runtime.runtime.commit_send_money(first));
    let second_runtime = std::sync::Arc::clone(&world);
    let second = std::thread::spawn(move || second_runtime.runtime.commit_send_money(second));
    let outcomes = [
        first.join().unwrap().unwrap(),
        second.join().unwrap().unwrap(),
    ];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| matches!(outcome, BankMutationCommitOutcome::Committed(_)))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    BankMutationCommitOutcome::Denied {
                        kind: bank_server::BankCommitDenialKind::ProductBasisStale,
                        stage: bank_server::BankCommitDenialStage::InvariantExecution,
                    }
                )
            })
            .count(),
        1
    );

    let final_admission = world
        .runtime
        .authorize_send_money(&actor, source, Default::default(), &request)
        .unwrap();
    let denial = BankOperationProposals::prepare_send_money(
        &world.runtime,
        final_admission,
        &key("post-concurrency-overspend"),
        &SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(4_001).unwrap(),
        },
    )
    .err()
    .expect("journal-derived remaining funds must deny the overspend");
    assert_eq!(
        denial,
        BankOperationProposalError::Invariant(BankProposalDenial::InsufficientFunds(source))
    );
}

#[test]
fn independent_transfers_derive_disjoint_ids_and_commit_across_world_generations() {
    let snapshot = funded_independent_sender_world();
    let identities = (1..=4)
        .map(|principal| {
            (
                principal,
                DynamicIdentity::new(&format!("independent-{principal}")),
            )
        })
        .collect::<Vec<_>>();
    let mut seed = BankWorldSeed::new(snapshot.clone());
    for (principal, identity) in &identities {
        seed = seed.principal(BankPrincipalSeed::enabled(
            id(BankPrincipalId::new, *principal),
            identity.external(),
        ));
    }
    let world = std::sync::Arc::new(runtime(seed));
    let request = request_scope();
    let first_actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&identities[0].1),
        &request,
    ))
    .unwrap();
    let second_actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&identities[1].1),
        &request,
    ))
    .unwrap();
    let first_source = snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap();
    let second_source = snapshot
        .primary_account(id(BankPrincipalId::new, 2))
        .unwrap();
    let first = prepare_send(
        &world,
        &request,
        &first_actor,
        first_source,
        3,
        "independent-first",
    );
    let first_identity = journal_identity(&first);
    let first = world.runtime.commit_send_money(first).unwrap();
    assert!(matches!(first, BankMutationCommitOutcome::Committed(_)));
    let second = prepare_send(
        &world,
        &request,
        &second_actor,
        second_source,
        4,
        "independent-second",
    );
    assert_ne!(first_identity, journal_identity(&second));
    let second = world.runtime.commit_send_money(second).unwrap();
    assert!(matches!(second, BankMutationCommitOutcome::Committed(_)));
}

#[test]
fn unrelated_world_growth_does_not_increase_send_projection_work() {
    let baseline = projection_work(funded_personal_world(), "baseline", 0);
    let enlarged = projection_work(
        funded_personal_world_with_unrelated_journals(64),
        "enlarged",
        64,
    );

    assert_eq!(enlarged, baseline);
    assert_eq!(baseline.reconstructive_scans(), 0);
    assert!(baseline.equality_lookups() > 0);
    assert!(baseline.adjacency_lists_read() > 0);
}

fn prepare_send(
    world: &crate::support::TestIdentityWorld,
    request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    actor: &bank_server::BankAuthenticatedPrincipal,
    source: bank_domain::model::AccountId,
    recipient: u64,
    operation_key: &str,
) -> bank_server::BankAuthorizedProposal<
    bank_domain::schema::SendMoneyOperation,
    SendMoney,
    bank_domain::schema::Account,
    bank_domain::model::AccountId,
> {
    let admission = world
        .runtime
        .authorize_send_money(actor, source, Default::default(), request)
        .unwrap();
    expect_send_proposal(
        BankOperationProposals::prepare_send_money(
            &world.runtime,
            admission,
            &key(operation_key),
            &SendMoney {
                from: source,
                recipient: id(BankPrincipalId::new, recipient),
                amount: bank_domain::model::Money::from_minor(100).unwrap(),
            },
        )
        .unwrap(),
    )
}

fn journal_identity(
    proposal: &bank_server::BankAuthorizedProposal<
        bank_domain::schema::SendMoneyOperation,
        SendMoney,
        bank_domain::schema::Account,
        bank_domain::model::AccountId,
    >,
) -> bank_domain::model::JournalEntryId {
    let [BankProposedEffect::AppendJournal(journal)] = proposal.invariant().effects() else {
        panic!("send must propose one journal");
    };
    journal.id()
}

fn projection_work(
    snapshot: bank_domain::proposals::BankSnapshot,
    identity: &str,
    unrelated_principals: usize,
) -> BankMutationProjectionWork {
    let owner_name = format!("{identity}-owner");
    let recipient_name = format!("{identity}-recipient");
    let employee_name = format!("{identity}-employee");
    let owner = DynamicIdentity::new(&owner_name);
    let recipient = DynamicIdentity::new(&recipient_name);
    let employee = DynamicIdentity::new(&employee_name);
    let mut seed = BankWorldSeed::new(snapshot.clone())
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
        ));
    for ordinal in 0..unrelated_principals {
        let identity_name = format!("{identity}-unrelated-{ordinal}");
        let unrelated_identity = DynamicIdentity::new(&identity_name);
        seed = seed.principal(BankPrincipalSeed::enabled(
            unrelated_principal(ordinal),
            unrelated_identity.external(),
        ));
    }
    let world = runtime(seed);
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
    let admission = world
        .runtime
        .authorize_send_money(&actor, source, Default::default(), &request)
        .unwrap();
    expect_send_proposal(
        BankOperationProposals::prepare_send_money(
            &world.runtime,
            admission,
            &key("projection-work"),
            &SendMoney {
                from: source,
                recipient: id(BankPrincipalId::new, 2),
                amount: Money::from_minor(1).unwrap(),
            },
        )
        .unwrap(),
    )
    .projection_work()
}
