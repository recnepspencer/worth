//! A foreign canceled scope cannot terminate the installed live subscription.
use super::*;
use crate::estate_capability_admission::fixture::{
    live_read_world, request_scope, INSTITUTION, OTHER_ACCOUNT,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveNextDenial, WorthQueryApplicationMutationOutcome,
};

#[test]
fn canceled_foreign_branch_is_refused_and_the_next_good_request_delivers() {
    let fixture = live_read_world("live-foreign-canceled-request");
    let owner = fixture.authenticate_executor();
    let teller = fixture.authenticate();
    let runtime = &fixture.runtime;
    let application = runtime.application_runtime();
    let scope = request_scope();
    let foreign = application
        .with_application_advancement(&scope, |phase| {
            runtime
                .application_program()
                .branches()
                .fork(runtime.application_program().current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let mut live = runtime
        .account_activity(OTHER_ACCOUNT)
        .as_principal(&owner)
        .subscribe(
            WorthQueryApplicationLiveControls::bounded(request_scope(), 16, 8, 2_048).unwrap(),
        )
        .unwrap();
    let cancellation = WorthQueryCancellationSource::new();
    let canceled_scope = WorthQueryRequestScope::new(
        Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
        cancellation.token(),
    );
    cancellation.cancel();
    let foreign_request = application
        .request(owner.external(), &canceled_scope)
        .on_branch(foreign);
    assert!(matches!(
        live.query.next(&foreign_request),
        Err(WorthQueryApplicationLiveNextDenial::ForeignBranch)
    ));
    let committed = runtime
        .mutate(crate::mutations::deposit(bank_domain::schema::Deposit {
            institution: INSTITUTION,
            account: OTHER_ACCOUNT,
            amount: bank_domain::model::Money::from_minor(1).unwrap(),
        }))
        .as_principal(&teller)
        .controls(crate::BankMutationControls::new(
            request_scope(),
            bank_domain::proposals::BankIdempotencyKey::new("live-foreign-probe-deposit").unwrap(),
        ))
        .execute()
        .unwrap();
    assert!(matches!(
        committed,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(matches!(
        live.poll(&owner, &request_scope()),
        BankAccountActivityLiveOutcome::Delivered(_)
    ));
}
