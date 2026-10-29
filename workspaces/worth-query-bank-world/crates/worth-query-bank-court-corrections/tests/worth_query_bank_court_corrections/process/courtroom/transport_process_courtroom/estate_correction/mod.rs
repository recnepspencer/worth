pub(crate) use worth_query_bank_testkit_courtroom::facade::transport_process_courtroom::estate_correction::*;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the real Docker identity provider and cold Bank/user-node processes"]
async fn closed_estate_freeze_corrects_until_its_actual_capability_is_revoked() {
    let world = TransportProcessWorld::start().await;
    world.authenticate_participants().await;
    let original = account(&world, world.primary_address, "fixture:100", "original").await;
    let foreign = account(&world, world.peer_address, "fixture:101", "foreign").await;
    assert_eq!(original.status, BankHttpAccountStatus::Open);

    let first = freeze_and_close(&world, "first-freeze").await;
    assert_status(
        &world,
        BankHttpAccountStatus::Frozen,
        &original,
        "first-frozen",
    )
    .await;
    let corrected = complete_and_close(
        &world,
        "restore-first",
        serde_json::json!({"operation":"correct_estate_freeze", "source":first}),
    )
    .await;
    assert_ne!(corrected, first, "correction publishes its own occurrence");
    assert_status(&world, BankHttpAccountStatus::Open, &original, "restored").await;

    let second = freeze_and_close(&world, "second-freeze").await;
    assert_ne!(
        second, first,
        "the later freeze is a distinct recorded source"
    );
    assert_status(
        &world,
        BankHttpAccountStatus::Frozen,
        &original,
        "second-frozen",
    )
    .await;
    complete_and_close(
        &world,
        "revoke-freeze-grant",
        serde_json::json!({"operation":"revoke_estate_capability", "estate":3, "grant":26}),
    )
    .await;
    let denied = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &create(
                "restore-revoked",
                serde_json::json!({
                    "operation":"correct_estate_freeze", "source":second,
                }),
            ),
        )
        .await,
    );
    assert!(
        matches!(denied,
        BankHttpMutationOutcome::Rejected { ref denial, .. }
        if denial.kind == BankHttpDenialKind::PermissionDenied),
        "fresh capability admission must refuse the historical correction: {denied:?}"
    );
    assert_status(
        &world,
        BankHttpAccountStatus::Frozen,
        &original,
        "revoked-frozen",
    )
    .await;
    assert_eq!(
        account(&world, world.peer_address, "fixture:101", "foreign-after").await,
        foreign,
        "estate correction and revocation preserve the unrelated account",
    );
    world.shutdown().await;
}
