use super::*;
use bank_server::{
    BankAccountActivityLiveOutcome, BankApplicationLiveCloseOutcome,
    BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankAuthorizationDenialKind,
};
use worth_query_host::facade::application_entry::{
    WorthQuerySourceCompletion as Completion, WorthQuerySourceOnlyAdvance as Advance,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

#[test]
fn owning_deposit_and_replay_deliver_the_original_retained_commit() {
    let fixture = ordinary_read_world("ordinary-retained-deposit", 0);
    let teller = fixture.authenticate(TELLER);
    let owner = fixture.authenticate(OWNER);
    let viewer = fixture.authenticate(fixture::VIEWER);
    let account = fixture.personal_account;
    let saved_view = fixture.world.runtime.account_activity(account);
    let before = saved_view
        .as_principal(&owner)
        .execute(BankReadControls::current(request_scope(), 128).unwrap())
        .expect("the saved view selects the current product")
        .rows()[0]
        .entries()
        .len();
    let deposit_scope = request_scope();
    let deposit_request = fixture.world.runtime.request(&teller, &deposit_scope);
    let history = std::num::NonZeroUsize::new(20_000).unwrap();
    let deposit = Deposit {
        institution: fixture.institution,
        account,
        amount: Money::from_minor(17).unwrap(),
    };
    let performed = fixture
        .world
        .runtime
        .mutate(mutations::deposit(deposit.clone()))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            request_scope(),
            key("retained-deposit"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("deposit construction: {cause:?}"));
    let Advance::Completed(performed) = performed.advance(&deposit_request, Some(history)) else {
        panic!("fresh retained deposit")
    };
    let Completion::Fresh {
        receipt,
        retained: Some(retained),
        ..
    } = performed.evidence()
    else {
        panic!("actual original observation")
    };
    assert_eq!(
        retained.selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    let at_commit = saved_view
        .as_principal(&owner)
        .retained(
            retained,
            BankReadControls::current(request_scope(), 128).unwrap(),
        )
        .expect("fresh security should admit read-after at the performed product");
    assert_eq!(at_commit.rows()[0].entries().len(), before + 1);
    let allowed_viewer = saved_view
        .as_principal(&viewer)
        .retained(
            retained,
            BankReadControls::current(request_scope(), 128).unwrap(),
        )
        .expect("the currently authorized viewer can read the retained publication");
    assert_eq!(allowed_viewer.rows()[0].entries().len(), before + 1);

    let mut live = saved_view
        .as_principal(&owner)
        .subscribe(WorthQueryApplicationLiveControls::bounded(request_scope(), 16, 8).unwrap())
        .expect("the saved view opens the same query as a live subscription");
    assert!(matches!(
        live.poll(&owner, &request_scope()),
        BankAccountActivityLiveOutcome::Pending
    ));

    let later = Deposit {
        amount: Money::from_minor(19).unwrap(),
        ..deposit.clone()
    };
    let later = fixture
        .world
        .runtime
        .mutate(mutations::deposit(later))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            deposit_scope.clone(),
            key("later-deposit"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("later construction: {cause:?}"));
    let Advance::Completed(later) = later.advance(&deposit_request, Some(history)) else {
        panic!("later fresh deposit")
    };
    assert!(later.close(&deposit_request).is_ok());
    let BankAccountActivityLiveOutcome::Delivered(update) = live.poll(&owner, &request_scope())
    else {
        panic!("the saved view must deliver the later matching deposit");
    };
    assert_eq!(update.result().entries()[0].amount().minor_units(), 19);
    assert!(matches!(
        live.close(),
        BankApplicationLiveCloseOutcome::Completed
    ));
    assert_eq!(
        account_activity(&fixture, &owner, account).len(),
        before + 2
    );
    let page = saved_view
        .as_principal(&owner)
        .page(BankReadControls::current(request_scope(), 128).unwrap())
        .expect("the saved view also delivers the current page");
    assert_eq!(page.rows()[0].entries().len(), before + 2);
    let still_exact = saved_view
        .as_principal(&owner)
        .retained(
            retained,
            BankReadControls::current(request_scope(), 128).unwrap(),
        )
        .expect("later publication cannot retarget the retained read");
    assert_eq!(still_exact.rows()[0].entries().len(), before + 1);

    let replay = fixture
        .world
        .runtime
        .mutate(mutations::deposit(deposit))
        .as_principal(&teller)
        .controls(BankMutationControls::new(
            request_scope(),
            key("retained-deposit"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("replay construction: {cause:?}"));
    let Advance::Progress(replay) = replay.advance(&deposit_request, Some(history)) else {
        panic!("native retained replay pending")
    };
    let Advance::Completed(replay) = replay.advance(&deposit_request, Some(history)) else {
        panic!("bounded original native replay delivery")
    };
    let Completion::Recovered {
        receipt: replay_receipt,
        retained: Some(replay_retained),
        ..
    } = replay.evidence()
    else {
        panic!("real replay observation, no handler result")
    };
    assert_eq!(
        replay_retained.selected_commit(),
        retained.selected_commit()
    );
    let replay_receipt = replay_receipt.clone();
    assert!(replay.close(&deposit_request).is_ok());
    let authorization = authorized_users(&fixture, &owner)
        .into_iter()
        .find(|user| user.principal() == principal_id(fixture::VIEWER))
        .expect("fixture viewer must have account access")
        .authorization();
    let scope = request_scope();
    let request = fixture.world.runtime.request(&owner, &scope);
    let operation = fixture
        .world
        .runtime
        .mutate(mutations::revoke_account_access(
            RevokeAccountAuthorization {
                account,
                authorization,
            },
        ))
        .as_principal(&owner)
        .controls(BankMutationControls::new(
            scope.clone(),
            key("retained-revoke-viewer"),
        ))
        .retain_operation()
        .unwrap_or_else(|cause| panic!("declared revoke: {cause:?}"));
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(done) =
        operation.advance(&request)
    else {
        panic!("fresh revoke")
    };
    assert!(matches!(
        done.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("revoke close: {cause:?}"));
    let denied = saved_view.as_principal(&viewer).retained(
        retained,
        BankReadControls::current(request_scope(), 128).unwrap(),
    );
    assert!(matches!(
        denied,
        Err(BankApplicationQueryDenial::Admission(error))
            if error.kind()
                == BankApplicationQueryAdmissionDenialKind::Authorization(
                    BankAuthorizationDenialKind::PermissionDenied
                )
    ));
    assert!(performed.close(&deposit_request).is_ok());
    let historical = saved_view
        .as_principal(&owner)
        .historical(
            &replay_receipt,
            std::num::NonZeroUsize::new(128).unwrap(),
            std::num::NonZeroUsize::new(8_192).unwrap(),
            &request_scope(),
        )
        .expect("a replay receipt may separately select bounded history after the lease drops");
    assert_eq!(historical.rows()[0].entries().len(), before + 1);
}
