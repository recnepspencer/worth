pub(crate) use worth_query_bank_worlds::facade::courtroom::transport_process_courtroom::recorded_correction::reapplication::*;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires the real Docker identity provider and cold Bank/user-node processes"]
async fn historical_money_reapplication_preserves_selection_across_response_loss_and_later_lineage()
{
    let world = TransportProcessWorld::start().await;
    world.authenticate_participants().await;
    let baseline = balances(&world, "baseline").await;
    // Each completed owner retains its own World occurrence while the chooser
    // and retries still need it. Scalar rows and tokens do not retain history.
    let (source, source_close) = send_retained(&world, "history-source", 30).await;
    send(&world, "history-unrelated-seven", 7).await;
    let u_close = reverse(&world, "history-u", &source).await;
    let (first_token, first_sources) = choose(&world, &source, "history-first").await;
    assert_eq!(first_sources, vec![source.clone()]);

    let request = reapply("history-lost-r", "history-r-key", &first_token, &source);
    drop(
        world
            .client
            .post(format!("http://{}/v1/mutations", world.peer_address))
            .json(&request)
            .send()
            .await
            .expect("discard response body after native publication"),
    );
    let (r, recovered, r_close) = finish(&world, "history-lost-r", None).await;
    assert!(!recovered);
    assert_balances(&world, baseline, 37, "first-reapplication").await;
    let (none_active_token, _) = choose(&world, &source, "history-none-active").await;
    let replay = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &reapply(
                "history-none-active-retry",
                "history-r-key",
                &none_active_token,
                &source,
            ),
        )
        .await,
    );
    let (replayed_r, recovered, next) =
        finish(&world, "history-none-active-retry", Some(replay)).await;
    assert!(recovered);
    assert_eq!(replayed_r, r);
    close(&world, "history-none-active-retry", next).await;

    let v_close = reverse(&world, "history-v", &r).await;
    send(&world, "history-unrelated-three", 3).await;
    let before = activities(&world, "before-older-u").await;
    let stale = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &reapply("history-stale", "history-stale-key", &first_token, &source),
        )
        .await,
    );
    assert!(
        matches!(&stale, BankHttpMutationOutcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Stale && denial.next_action == bank_http_adapter::BankHttpNextAction::Refresh),
        "an advanced same-branch chooser must require refreshed history: {stale:?}"
    );
    let (token, alternatives) = choose(&world, &source, "history-two-pages").await;
    assert_eq!(alternatives.len(), 2);
    assert!(alternatives.contains(&source) && alternatives.contains(&r));
    let foreign = forwarded(
        node_mutation(
            &world.client,
            world.primary_address,
            &reapply("history-foreign", "history-foreign-key", &token, &source),
        )
        .await,
    );
    assert!(
        matches!(foreign, BankHttpMutationOutcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    assert_eq!(activities(&world, "denied-unchanged").await, before);

    let reapplied = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &reapply("history-older-u", "history-older-u-key", &token, &source),
        )
        .await,
    );
    let (r2, recovered, r2_close) = finish(&world, "history-older-u", Some(reapplied)).await;
    assert!(!recovered);
    assert_ne!(r2, source);
    assert_ne!(r2, r);
    assert_balances(&world, baseline, 40, "older-u-reapplied").await;
    let after = activities(&world, "after-older-u").await;
    assert_preserved(&before.0, &after.0);
    assert_preserved(&before.1, &after.1);
    assert_eq!(after.0.len(), before.0.len() + 1);
    assert_eq!(after.1.len(), before.1.len() + 1);

    let v2_close = reverse(&world, "history-v2", &r2).await;
    assert_balances(&world, baseline, 10, "later-lineage").await;
    let later = activities(&world, "later-journals").await;
    let (current_token, _) = choose(&world, &source, "history-later-pages").await;
    let replay = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &reapply(
                "history-later-retry",
                "history-older-u-key",
                &current_token,
                &source,
            ),
        )
        .await,
    );
    let (same_r2, recovered, next) = finish(&world, "history-later-retry", Some(replay)).await;
    assert!(recovered);
    assert_eq!(
        same_r2, r2,
        "later active reversal cannot replace recorded selected/active command meaning"
    );
    close(&world, "history-later-retry", next).await;
    let drift = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &reapply(
                "history-selection-drift",
                "history-older-u-key",
                &current_token,
                &r,
            ),
        )
        .await,
    );
    assert!(
        matches!(drift, BankHttpMutationOutcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Stale)
    );
    assert_eq!(activities(&world, "replayed-journals").await, later);
    assert_balances(&world, baseline, 10, "replayed-balance").await;
    for (custody, close_sequence) in [
        ("history-source", source_close),
        ("history-u", u_close),
        ("history-lost-r", r_close),
        ("history-v", v_close),
        ("history-older-u", r2_close),
        ("history-v2", v2_close),
    ] {
        close(&world, custody, close_sequence).await;
    }
    world.shutdown().await;
}
