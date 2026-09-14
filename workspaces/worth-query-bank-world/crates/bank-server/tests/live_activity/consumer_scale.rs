use super::*;

#[test]
fn live_consumer_fanout_keeps_each_delivery_free_of_canonical_work() {
    const CONSUMER_COUNT: usize = 32;

    let fixture = ordinary_read_world("live-consumer-canonical-scale", 0);
    let owner = fixture.authenticate(OWNER);
    let teller = fixture.authenticate(TELLER);
    let mut leases = (0..CONSUMER_COUNT)
        .map(|_| {
            fixture
                .world
                .runtime
                .account_activity(fixture.personal_account)
                .as_principal(&owner)
                .subscribe(live_controls())
                .expect("each bounded authenticated live consumer should open")
        })
        .collect::<Vec<_>>();

    commit_deposit(
        &fixture,
        &teller,
        fixture.personal_account,
        "live-consumer-canonical-scale-deposit",
    );

    let mut expected_publication_work = None;
    for lease in &mut leases {
        let BankAccountActivityLiveOutcome::Delivered(update) =
            lease.poll(&owner, &request_scope())
        else {
            panic!("each retained consumer must receive the matching commit")
        };
        let receipt = update.receipt();
        let inspection = receipt.inspect();
        let publication_work = (
            inspection.publication_canonical_entries(),
            inspection.publication_sha256_compression_blocks(),
            inspection.publication_identity_text_materializations(),
        );
        assert!(inspection.terminal_resources_released());
        assert_eq!(
            *expected_publication_work.get_or_insert(publication_work),
            publication_work
        );
    }
    for lease in leases {
        assert!(matches!(
            lease.close(),
            bank_server::BankApplicationLiveCloseOutcome::Completed
        ));
    }
}
