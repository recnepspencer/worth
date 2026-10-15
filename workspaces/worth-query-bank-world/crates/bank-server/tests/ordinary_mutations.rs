use worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
#[path = "ordinary_mutations/authorization_time.rs"]
mod authorization_time;
#[path = "ordinary_mutations/estate_operations.rs"]
mod estate_operations;
#[allow(
    dead_code,
    reason = "the shared read fixture has discovery-only helpers exercised by its owning test binary"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
#[path = "ordinary_mutations/preconditions.rs"]
mod preconditions;
#[path = "ordinary_mutations/publication.rs"]
mod publication;
mod support;

use std::time::{Duration, Instant};

use bank_domain::model::{AccountName, BusinessId, CustomerRole, InstitutionId, Money};
use bank_domain::proposals::BankIdempotencyKey;
use bank_domain::schema::{
    ApplyOpeningFunding, ApprovePayment, CreateBusinessAccount, CreatePersonalAccount, Deposit,
    GrantAccountAuthorization, InitiateBusinessPayment, PostingPurpose, RejectPayment,
    ReversalReason, ReverseJournal, RevokeAccountAuthorization, SendMoney, Withdraw,
};
use bank_server::{mutations, queries, BankMutationControls, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitTerminalKind;

use fixture::{ordinary_read_world, principal_id, APPROVER, OWNER, RECIPIENT, STRANGER, TELLER};
use support::request_scope;

#[path = "ordinary_mutations/money_families.rs"]
mod money_families;
#[path = "ordinary_mutations/owning_deposit.rs"]
mod owning_deposit;
#[path = "ordinary_mutations/program_families.rs"]
mod program_families;
#[path = "ordinary_mutations/retained_publication.rs"]
mod retained_publication;
#[path = "ordinary_mutations/transfer_posting_oracle.rs"]
mod transfer_posting_oracle;

#[test]
fn public_mutation_controls_preserve_interruptions_permissions_and_intent_drift() {
    use worth_query_host::facade::application_entry::{
        WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
        WorthQuerySourceOnlyDenial as Denial,
    };
    let fixture = ordinary_read_world("ordinary-mutation-controls", 0);
    let owner = fixture.authenticate(OWNER);
    let stranger = fixture.authenticate(STRANGER);
    let send = send_for(&fixture);
    for cancelled in [true, false] {
        let cancellation = WorthQueryCancellationSource::new();
        if cancelled {
            cancellation.cancel();
        }
        let scope = WorthQueryRequestScope::new(
            Some(if cancelled {
                Instant::now() + Duration::from_secs(60)
            } else {
                Instant::now() - Duration::from_secs(1)
            }),
            cancellation.token(),
        );
        let operation = fixture
            .world
            .runtime
            .mutate(mutations::send_money(send.clone()))
            .as_principal(&owner)
            .controls(BankMutationControls::new(
                scope.clone(),
                key(if cancelled { "cancelled" } else { "expired" }),
            ))
            .retain_operation()
            .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"));
        let request = fixture.world.runtime.request(&owner, &scope);
        let Advance::Refused {
            operation,
            denial: Denial::Execution(WorthQueryAdvancementDenial::Interrupted(interruption)),
        } = operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
        else {
            panic!("exact request interruption")
        };
        assert_eq!(
            interruption,
            if cancelled {
                WorthQueryManagedComputationInterruption::Cancelled
            } else {
                WorthQueryManagedComputationInterruption::DeadlineExceeded
            }
        );
        let _runtime = operation.into_runtime_for_cleanup();
    }
    let scope = request_scope();
    let request = fixture.world.runtime.request(&stranger, &scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&stranger)
        .controls(BankMutationControls::new(scope.clone(), key("denied")))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"));
    let Advance::Refused {
        operation,
        denial: Denial::Mutation(denial),
    } = operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("native permission refusal")
    };
    assert!(matches!(denial.kind(), worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::Authorization(worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::PermissionDenied))));
    let _runtime = operation.into_runtime_for_cleanup();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&owner, &scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::send_money(send.clone()))
        .as_principal(&owner)
        .controls(BankMutationControls::new(scope.clone(), key("drift")))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"));
    let Advance::Completed(completed) =
        operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("fresh send")
    };
    let Completion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("fresh evidence")
    };
    assert!(receipt.emitted_effect_count() > 0);
    assert!(completed.close(&request).is_ok());
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::send_money(SendMoney {
            amount: Money::from_minor(26).unwrap(),
            ..send
        }))
        .as_principal(&owner)
        .controls(BankMutationControls::new(scope.clone(), key("drift")))
        .retain_operation()
        .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"));
    let Advance::Terminal(terminal) =
        operation.advance(&request, std::num::NonZeroUsize::new(20_000).unwrap())
    else {
        panic!("intent drift terminal")
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    ));
    assert!(terminal.close(&request).is_ok());
}

fn pending_payments(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::PaymentSummary> {
    let result = fixture
        .world
        .runtime
        .query(queries::pending_payments())
        .as_principal(principal)
        .controls(read_controls())
        .execute()
        .expect("pending payments must be readable");
    result.rows().to_vec()
}

fn authorized_users(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
) -> Vec<bank_domain::reads::AuthorizedAccountUser> {
    let result = fixture
        .world
        .runtime
        .query(queries::account_authorized_users(fixture.personal_account))
        .as_principal(principal)
        .controls(read_controls())
        .execute()
        .expect("authorized users must be readable");
    let mut rows = result.into_rows();
    assert_eq!(rows.len(), 1);
    rows.pop().unwrap().into_users()
}

fn account_activity(
    fixture: &fixture::OrdinaryReadFixture,
    principal: &bank_server::BankAuthenticatedPrincipal,
    account: bank_domain::model::AccountId,
) -> Vec<bank_domain::reads::AccountActivityItem> {
    let request = request_scope();
    fixture
        .world
        .runtime
        .account_activity(account)
        .as_principal(principal)
        .execute(BankReadControls::current(request, 128, 8_192).unwrap())
        .expect("account activity must be readable")
        .rows()[0]
        .entries()
        .to_vec()
}

fn read_controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 128, 10_000).unwrap()
}

fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).unwrap()
}

fn send_for(fixture: &fixture::OrdinaryReadFixture) -> SendMoney {
    SendMoney {
        from: fixture.personal_account,
        recipient: principal_id(RECIPIENT),
        amount: Money::from_minor(25).unwrap(),
    }
}

#[path = "ordinary_mutations/posting_lease.rs"]
mod posting_lease;

#[path = "ordinary_mutations/owning_accounts.rs"]
mod owning_accounts;
