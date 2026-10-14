use std::sync::Arc;

use bank_domain::model::{BankPrincipalId, Money};
use bank_domain::proposals::BankProposalDenial;
use bank_domain::schema::SendMoney;
use bank_server::{mutations, BankMutationControls, BankPrincipalSeed, BankWorldSeed};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQuerySourceCompletion as Completion,
    WorthQuerySourceOnlyAdvance as Advance,
};

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
            let scope = request_scope();
            let request = world.runtime.request(&actor, &scope);
            let operation = world
                .runtime
                .mutate(mutations::send_money(input))
                .as_principal(&actor)
                .controls(controls(operation_key))
                .retain_operation()
                .unwrap_or_else(|(_, cause)| panic!("send construction: {cause:?}"));
            match operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap()) {
                Advance::Completed(done) => {
                    let Completion::Fresh { result, .. } = done.evidence() else {
                        panic!("fresh send evidence")
                    };
                    let journal = result.journal;
                    assert!(done.close(&request).is_ok());
                    Ok(journal)
                }
                Advance::Terminal(terminal) => {
                    let Some(WorthQueryApplicationMutationOutcome::DomainDenied(denial)) =
                        terminal.native_outcome()
                    else {
                        panic!("native domain refusal")
                    };
                    let denial = denial.clone();
                    assert!(terminal.close(&request).is_ok());
                    Err(denial)
                }
                _ => panic!("unexpected transfer custody"),
            }
        })
    };
    let outcomes = [
        run("program-concurrent-first").join().unwrap(),
        run("program-concurrent-second").join().unwrap(),
    ];
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| { matches!(outcome, Ok(_)) })
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    Err(BankProposalDenial::InsufficientFunds(account)) if *account == source
                )
            })
            .count(),
        1
    );

    let scope = request_scope();
    let request = world.runtime.request(&actor, &scope);
    let operation = world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(4_001).unwrap(),
        }))
        .as_principal(&actor)
        .controls(controls("program-post-concurrency-overspend"))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("send construction: {cause:?}"));
    let Advance::Terminal(terminal) =
        operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("native overspend denial")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            BankProposalDenial::InsufficientFunds(account)
        )) if *account == source
    ));
    assert!(terminal.close(&request).is_ok());
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
    assert_ne!(first, second);
}

fn execute_send(
    world: &crate::support::TestIdentityWorld,
    identity: &DynamicIdentity,
    source: bank_domain::model::AccountId,
    recipient: u64,
    operation_key: &str,
) -> bank_domain::model::JournalEntryId {
    let request = request_scope();
    let actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(identity),
        &request,
    ))
    .unwrap();
    let operation = world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, recipient),
            amount: Money::from_minor(100).unwrap(),
        }))
        .as_principal(&actor)
        .controls(controls(operation_key))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("send construction: {cause:?}"));
    let fresh = world.runtime.request(&actor, &request);
    let Advance::Completed(done) =
        operation.advance(&fresh, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("independent native send")
    };
    let Completion::Fresh { result, .. } = done.evidence() else {
        panic!("fresh journal")
    };
    let journal = result.journal;
    assert!(done.close(&fresh).is_ok());
    journal
}

fn controls(operation_key: &str) -> BankMutationControls {
    BankMutationControls::new(request_scope(), key(operation_key))
}
