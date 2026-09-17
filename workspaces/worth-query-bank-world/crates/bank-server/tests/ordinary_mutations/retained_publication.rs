use super::*;
use bank_server::{
    BankAccountActivityLiveOutcome, BankApplicationLiveCloseOutcome,
    BankApplicationQueryAdmissionDenialKind, BankApplicationQueryDenial,
    BankAuthorizationDenialKind,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRetainedMutationOutcome;
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

#[test]
fn opted_in_deposit_carries_exact_read_after_and_replay_cannot_mint_it() {
    let fixture = ordinary_read_world("ordinary-retained-deposit", 0);
    let teller = fixture.authenticate(TELLER);
    let owner = fixture.authenticate(OWNER);
    let viewer = fixture.authenticate(fixture::VIEWER);
    let account = fixture.personal_account;
    let saved_view = fixture.world.runtime.account_activity(account);
    let before = saved_view
        .as_principal(&owner)
        .execute(BankReadControls::current(request_scope(), 128, 8_192).unwrap())
        .expect("the saved view selects the current product")
        .rows()[0]
        .entries()
        .len();
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
        .execute_retained()
        .expect("an opted-in deposit should reach publication");
    let WorthQueryApplicationRetainedMutationOutcome::Committed {
        receipt, retained, ..
    } = performed
    else {
        panic!("the first deposit must perform with a retained successor");
    };
    assert_eq!(
        retained.selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    let at_commit = saved_view
        .as_principal(&owner)
        .retained(
            &retained,
            BankReadControls::current(request_scope(), 128, 8_192).unwrap(),
        )
        .expect("fresh security should admit read-after at the performed product");
    assert_eq!(at_commit.rows()[0].entries().len(), before + 1);
    let allowed_viewer = saved_view
        .as_principal(&viewer)
        .retained(
            &retained,
            BankReadControls::current(request_scope(), 128, 8_192).unwrap(),
        )
        .expect("the currently authorized viewer can read the retained publication");
    assert_eq!(allowed_viewer.rows()[0].entries().len(), before + 1);

    let mut live = saved_view
        .as_principal(&owner)
        .subscribe(
            WorthQueryApplicationLiveControls::bounded(request_scope(), 16, 8, 2_048).unwrap(),
        )
        .expect("the saved view opens the same query as a live subscription");
    assert!(matches!(
        live.poll(&owner, &request_scope()),
        BankAccountActivityLiveOutcome::Pending
    ));

    let later = Deposit {
        amount: Money::from_minor(19).unwrap(),
        ..deposit.clone()
    };
    assert!(matches!(
        execute!(fixture, teller, mutations::deposit(later), "later-deposit"),
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ));
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
        .page(BankReadControls::current(request_scope(), 128, 8_192).unwrap())
        .expect("the saved view also delivers the current page");
    assert_eq!(page.rows()[0].entries().len(), before + 2);
    let still_exact = saved_view
        .as_principal(&owner)
        .retained(
            &retained,
            BankReadControls::current(request_scope(), 128, 8_192).unwrap(),
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
        .execute_retained()
        .expect("equivalent idempotent replay should return its prior outcome");
    let WorthQueryApplicationRetainedMutationOutcome::Other(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replay_receipt),
    ) = &replay
    else {
        panic!("equivalent replay must return only a descriptive receipt");
    };
    assert!(replay.retained_read().is_none());
    let authorization = authorized_users(&fixture, &owner)
        .into_iter()
        .find(|user| user.principal() == principal_id(fixture::VIEWER))
        .expect("fixture viewer must have account access")
        .authorization();
    assert!(matches!(
        execute!(
            fixture,
            owner,
            mutations::revoke_account_access(RevokeAccountAuthorization {
                account,
                authorization,
            }),
            "retained-revoke-viewer"
        ),
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
    ));
    let denied = saved_view.as_principal(&viewer).retained(
        &retained,
        BankReadControls::current(request_scope(), 128, 8_192).unwrap(),
    );
    assert!(matches!(
        denied,
        Err(BankApplicationQueryDenial::Admission(error))
            if error.kind()
                == BankApplicationQueryAdmissionDenialKind::Authorization(
                    BankAuthorizationDenialKind::PermissionDenied
                )
    ));
    drop(retained);
    let historical = saved_view
        .as_principal(&owner)
        .historical(
            replay_receipt,
            std::num::NonZeroUsize::new(128).unwrap(),
            std::num::NonZeroUsize::new(8_192).unwrap(),
            &request_scope(),
        )
        .expect("a replay receipt may separately select bounded history after the lease drops");
    assert_eq!(historical.rows()[0].entries().len(), before + 1);
}
