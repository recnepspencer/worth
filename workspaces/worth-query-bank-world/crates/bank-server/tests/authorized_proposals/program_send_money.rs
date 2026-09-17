use std::sync::Arc;

use bank_domain::model::{BankPrincipalId, Money};
use bank_domain::proposals::BankProposalDenial;
use bank_domain::schema::SendMoney;
use bank_server::{BankPrincipalSeed, BankWorldSeed};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

use super::fixture::{funded_independent_sender_world, funded_personal_world, id, key};
use crate::support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

#[test]
fn concurrent_program_transfers_cannot_overspend_one_account() {
    let snapshot = funded_personal_world();
    let owner = DynamicIdentity::new("program-concurrent-owner");
    let recipient = DynamicIdentity::new("program-concurrent-recipient");
    let unrelated = DynamicIdentity::new("program-concurrent-unrelated");
    let world = Arc::new(runtime(
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
                unrelated.external(),
            )),
    ));
    let actor = Arc::new(
        block_on(world.runtime.authenticate_with(
            &world.authentication,
            CausalCredential::for_identity(&owner),
            &request_scope(),
        ))
        .unwrap(),
    );
    let source = snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap();
    let input = SendMoney {
        from: source,
        recipient: id(BankPrincipalId::new, 2),
        amount: Money::from_minor(6_000).unwrap(),
    };

    let run = |operation_key: &'static str| {
        let world = Arc::clone(&world);
        let actor = Arc::clone(&actor);
        let input = input.clone();
        std::thread::spawn(move || {
            let request = request_scope();
            world
                .runtime
                .request(&actor, &request)
                .mutate(input)
                .idempotency(&key(operation_key))
                .execute_in_program(world.runtime.application_program())
        })
    };
    let outcomes = [
        run("program-concurrent-first").join().unwrap(),
        run("program-concurrent-second").join().unwrap(),
    ];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
                )
            })
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    Ok(WorthQueryApplicationMutationOutcome::DomainDenied(
                        BankProposalDenial::InsufficientFunds(account)
                    )) if *account == source
                )
            })
            .count(),
        1
    );

    let request = request_scope();
    let denial = world
        .runtime
        .request(&actor, &request)
        .mutate(SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(4_001).unwrap(),
        })
        .idempotency(&key("program-post-concurrency-overspend"))
        .execute_in_program(world.runtime.application_program())
        .unwrap();
    assert!(matches!(
        denial,
        WorthQueryApplicationMutationOutcome::DomainDenied(
            BankProposalDenial::InsufficientFunds(account)
        ) if account == source
    ));
}

#[test]
fn independent_program_transfers_commit_across_world_generations() {
    let snapshot = funded_independent_sender_world();
    let identities = (1..=4)
        .map(|principal| DynamicIdentity::new(&format!("program-independent-{principal}")))
        .collect::<Vec<_>>();
    let mut seed = BankWorldSeed::new(snapshot.clone());
    for (ordinal, identity) in identities.iter().enumerate() {
        seed = seed.principal(BankPrincipalSeed::enabled(
            id(BankPrincipalId::new, (ordinal + 1) as u64),
            identity.external(),
        ));
    }
    let world = runtime(seed);
    let first_source = snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap();
    let second_source = snapshot
        .primary_account(id(BankPrincipalId::new, 2))
        .unwrap();
    let first = execute_send(
        &world,
        &identities[0],
        first_source,
        3,
        "program-independent-first",
    );
    let second = execute_send(
        &world,
        &identities[1],
        second_source,
        4,
        "program-independent-second",
    );
    let (
        WorthQueryApplicationMutationOutcome::Committed {
            result: first_result,
            ..
        },
        WorthQueryApplicationMutationOutcome::Committed {
            result: second_result,
            ..
        },
    ) = (first, second)
    else {
        panic!("both independent program transfers must commit");
    };
    assert_ne!(first_result.journal, second_result.journal);
}

fn execute_send(
    world: &crate::support::TestIdentityWorld,
    identity: &DynamicIdentity,
    source: bank_domain::model::AccountId,
    recipient: u64,
    operation_key: &str,
) -> WorthQueryApplicationMutationOutcome<
    BankProposalDenial,
    bank_domain::schema::MoneyMovementResult,
> {
    let request = request_scope();
    let actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(identity),
        &request,
    ))
    .unwrap();
    world
        .runtime
        .request(&actor, &request)
        .mutate(SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, recipient),
            amount: Money::from_minor(100).unwrap(),
        })
        .idempotency(&key(operation_key))
        .execute_in_program(world.runtime.application_program())
        .unwrap()
}
