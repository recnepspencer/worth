use bank_domain::model::{AccountName, BankPrincipalId, InstitutionId, Money};
use bank_domain::schema::{CreatePersonalAccount, SendMoney};
use bank_server::{mutations, BankMutationControls, BankPrincipalSeed, BankWorldSeed};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind;
use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

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
    let operation = world
        .runtime
        .mutate(mutations::create_personal_account(CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, 1),
            display_name: AccountName::new("unauthorized").unwrap(),
        }))
        .as_principal(&owner_actor)
        .controls(BankMutationControls::new(
            request.clone(),
            key("customer-cannot-open"),
        ))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("declared creation: {cause:?}"));
    let fresh = world.runtime.request(&owner_actor, &request);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
        operation: _operation,
        denial:
            worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
                creation_denial,
            ),
    } = operation.advance(&fresh)
    else {
        panic!("customer cannot substitute teller")
    };
    assert!(matches!(creation_denial.kind(), WorthQueryApplicationRequestMutationDenialKind::Authorization(worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied))));
    let send_denial = world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            from: source,
            recipient: id(BankPrincipalId::new, 2),
            amount: Money::from_minor(1).unwrap(),
        }))
        .as_principal(&other_actor)
        .controls(BankMutationControls::new(
            request.clone(),
            key("non-owner-cannot-send"),
        ))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("send construction: {cause:?}"));
    let fresh = world.runtime.request(&other_actor, &request);
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Refused {
        operation,
        denial:
            worth_query_host::facade::application_entry::WorthQuerySourceOnlyDenial::Mutation(
                send_denial,
            ),
    } = send_denial.advance(&fresh, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("native non-owner refusal")
    };
    let _runtime = operation.into_runtime_for_cleanup();
    assert!(matches!(
        send_denial.kind(),
        WorthQueryApplicationRequestMutationDenialKind::Authorization(
            worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied)
        )
    ));
}
