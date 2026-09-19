use bank_domain::model::{AccountName, BankPrincipalId, InstitutionId, Money};
use bank_domain::schema::{CreatePersonalAccount, SendMoney};
use bank_server::{BankPrincipalSeed, BankWorldSeed};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind;

use super::fixture::{funded_personal_world, id, key};
use crate::support::{block_on, request_scope, runtime, CausalCredential, DynamicIdentity};

#[test]
fn program_entry_denies_unauthorized_account_mutations() {
    let snapshot = funded_personal_world();
    let owner = DynamicIdentity::new("owner");
    let other = DynamicIdentity::new("other");
    let employee = DynamicIdentity::new("employee");
    let world = runtime(
        BankWorldSeed::new(snapshot.clone())
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 1),
                owner.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 2),
                other.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                id(BankPrincipalId::new, 3),
                employee.external(),
            )),
    );
    let request = request_scope();
    let owner_actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&owner),
        &request,
    ))
    .unwrap();
    let other_actor = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&other),
        &request,
    ))
    .unwrap();
    let source = snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap();
    let creation_denial = world
        .runtime
        .request(&owner_actor, &request)
        .mutate(CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 1),
            display_name: AccountName::new("unauthorized").unwrap(),
        })
        .idempotency(&key("customer-cannot-open"))
        .execute_in_program(world.runtime.application_program())
        .expect_err("customer role cannot substitute teller authority");
    assert!(matches!(
        creation_denial.kind(),
        WorthQueryApplicationRequestMutationDenialKind::Authorization
    ));
    let send_denial = world
        .runtime
        .request(&other_actor, &request)
        .mutate(SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(1).unwrap(),
        })
        .idempotency(&key("non-owner-cannot-send"))
        .execute_in_program(world.runtime.application_program())
        .expect_err("non-owner must be denied");
    assert!(matches!(
        send_denial.kind(),
        WorthQueryApplicationRequestMutationDenialKind::Authorization
    ));
}
