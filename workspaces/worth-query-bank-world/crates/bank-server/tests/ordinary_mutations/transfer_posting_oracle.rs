use bank_domain::model::{AccountId, BankPrincipalId, BankSnapshotVersion};
use bank_domain::proposals::{
    BankOperationScopeBinding, BankOperationScopeEntityBinding, BankOperationScopeSchemaBinding,
    BankProposalEngine, BankSnapshotBuilder,
};
use bank_domain::schema::AccountStatus;
use bank_server::{BankPrincipalSeed, BankWorldSeed};

use super::*;
use crate::support::{block_on, runtime, CausalCredential, DynamicIdentity};

#[test]
fn public_transfer_posts_thirty_once_from_one_hundred_to_zero() {
    let institution = InstitutionId::new(1).unwrap();
    let owner_id = BankPrincipalId::new(1).unwrap();
    let recipient_id = BankPrincipalId::new(2).unwrap();
    let source = AccountId::new(10).unwrap();
    let destination = AccountId::new(11).unwrap();
    let owner_identity = DynamicIdentity::new("transfer-oracle-owner");
    let recipient_identity = DynamicIdentity::new("transfer-oracle-recipient");
    let snapshot = BankSnapshotBuilder::new(BankSnapshotVersion::new(1).unwrap())
        .institution(institution)
        .principal(owner_id)
        .principal(recipient_id)
        .institution_cash_account(AccountId::new(1).unwrap(), institution)
        .personal_account(
            source,
            institution,
            owner_id,
            AccountName::new("Source").unwrap(),
            AccountStatus::Open,
        )
        .personal_account(
            destination,
            institution,
            recipient_id,
            AccountName::new("Destination").unwrap(),
            AccountStatus::Open,
        )
        .build()
        .unwrap();
    let snapshot = BankProposalEngine::prepare_opening_funding(
        &snapshot,
        BankOperationScopeBinding::new(
            1,
            BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
            "transfer-oracle-seed",
            BankOperationScopeEntityBinding::new(0, 1, 1),
            BankOperationScopeEntityBinding::new(0, 3, 1),
        ),
        &key("transfer-oracle-seed"),
        &ApplyOpeningFunding {
            institution,
            account: source,
            amount: Money::from_minor(100).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone();
    let world = runtime(
        BankWorldSeed::new(snapshot)
            .principal(BankPrincipalSeed::enabled(
                owner_id,
                owner_identity.external(),
            ))
            .principal(BankPrincipalSeed::enabled(
                recipient_id,
                recipient_identity.external(),
            )),
    );
    let owner = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&owner_identity),
        &request_scope(),
    ))
    .unwrap();
    let recipient = block_on(world.runtime.authenticate_with(
        &world.authentication,
        CausalCredential::for_identity(&recipient_identity),
        &request_scope(),
    ))
    .unwrap();
    let balance = |account, principal: &bank_server::BankAuthenticatedPrincipal| {
        world
            .runtime
            .query(queries::account_summary(account))
            .as_principal(principal)
            .controls(read_controls())
            .execute()
            .unwrap()
            .into_rows()[0]
            .current_balance()
            .minor_units()
    };
    assert_eq!(
        (balance(source, &owner), balance(destination, &recipient)),
        (100, 0)
    );
    let before_source = world
        .runtime
        .account_activity(source)
        .as_principal(&owner)
        .execute(read_controls())
        .unwrap()
        .rows()[0]
        .entries()
        .to_vec();
    let before_destination = world
        .runtime
        .account_activity(destination)
        .as_principal(&recipient)
        .execute(read_controls())
        .unwrap()
        .rows()[0]
        .entries()
        .to_vec();
    let send = SendMoney {
        from: source,
        recipient: recipient_id,
        amount: Money::from_minor(30).unwrap(),
    };
    let commit = |send| {
        world
            .runtime
            .mutate(mutations::send_money(send))
            .as_principal(&owner)
            .controls(BankMutationControls::new(
                request_scope(),
                key("transfer-thirty"),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("transfer construction: {cause:?}"))
    };
    let scope = request_scope();
    let request = world.runtime.request(&owner, &scope);
    let history = std::num::NonZeroUsize::new(20_000).unwrap();
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(done) =
        commit(send.clone()).advance(&request, Some(history))
    else {
        panic!("fresh transfer")
    };
    assert!(matches!(
        done.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
    ));
    assert!(done.close(&request).is_ok());
    assert_eq!(
        (balance(source, &owner), balance(destination, &recipient)),
        (70, 30)
    );
    let source_entries = world
        .runtime
        .account_activity(source)
        .as_principal(&owner)
        .execute(read_controls())
        .unwrap()
        .rows()[0]
        .entries()
        .to_vec();
    let destination_entries = world
        .runtime
        .account_activity(destination)
        .as_principal(&recipient)
        .execute(read_controls())
        .unwrap()
        .rows()[0]
        .entries()
        .to_vec();
    assert_eq!(source_entries.len(), before_source.len() + 1);
    assert_eq!(destination_entries.len(), before_destination.len() + 1);
    assert_eq!(
        source_entries.last().unwrap().purpose(),
        PostingPurpose::Transfer
    );
    assert_eq!(
        destination_entries.last().unwrap().purpose(),
        PostingPurpose::Transfer
    );
    assert_eq!(source_entries.last().unwrap().amount().minor_units(), -30);
    assert_eq!(
        destination_entries.last().unwrap().amount().minor_units(),
        30
    );
    assert_eq!(
        source_entries.last().unwrap().journal(),
        destination_entries.last().unwrap().journal()
    );
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Progress(replay) =
        commit(send.clone()).advance(&request, Some(history))
    else {
        panic!("replay retains original commit delivery")
    };
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(replay) =
        replay.advance(&request, Some(history))
    else {
        panic!("original historical delivery")
    };
    assert!(matches!(
        replay.evidence(),
        worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Recovered { .. }
    ));
    assert!(replay.close(&request).is_ok());
    let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Terminal(drift) =
        commit(SendMoney {
            amount: Money::from_minor(31).unwrap(),
            ..send
        })
        .advance(&request, Some(history))
    else {
        panic!("native intent drift")
    };
    assert!(matches!(
        drift.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    ));
    assert!(drift.close(&request).is_ok());
    assert_eq!(
        (balance(source, &owner), balance(destination, &recipient)),
        (70, 30)
    );
    assert_eq!(
        world
            .runtime
            .account_activity(source)
            .as_principal(&owner)
            .execute(read_controls())
            .unwrap()
            .rows()[0]
            .entries(),
        source_entries,
    );
    assert_eq!(
        world
            .runtime
            .account_activity(destination)
            .as_principal(&recipient)
            .execute(read_controls())
            .unwrap()
            .rows()[0]
            .entries(),
        destination_entries,
    );
}
