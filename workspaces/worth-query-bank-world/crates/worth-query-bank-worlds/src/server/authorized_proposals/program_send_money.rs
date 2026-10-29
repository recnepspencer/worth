pub use std::sync::Arc;

pub use bank_domain::model::{BankPrincipalId, Money};
pub use bank_domain::proposals::BankProposalDenial;
pub use bank_domain::schema::SendMoney;
pub use bank_server::{mutations, BankMutationControls, BankPrincipalSeed, BankWorldSeed};
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQuerySourceCompletion as Completion,
    WorthQuerySourceOnlyAdvance as Advance,
};

pub use super::fixture::{funded_independent_sender_world, funded_personal_world, id, key};
pub use crate::facade::server::support::{
    block_on, request_scope, runtime, CausalCredential, DynamicIdentity,
};

pub fn execute_send(
    world: &crate::facade::server::support::TestIdentityWorld,
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
        .unwrap_or_else(|cause| panic!("send construction: {cause:?}"));
    let fresh = world.runtime.request(&actor, &request);
    let Advance::Completed(done) =
        operation.advance(&fresh, Some(std::num::NonZeroUsize::new(20_000).unwrap()))
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

pub fn controls(operation_key: &str) -> BankMutationControls {
    BankMutationControls::new(request_scope(), key(operation_key))
}
