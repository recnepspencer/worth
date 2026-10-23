//! One real process journey keeps HTTP selection separate from native lineage truth.
use super::*;
use bank_http_adapter::BankHttpCorrectionHistoryOutcome;
use bank_user_node::BankUserNodeCorrectionHistoryOutcome;

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

fn reapply(
    custody: &str,
    key: &str,
    token: &str,
    source: &BankHttpCorrectionSource,
) -> serde_json::Value {
    create(
        custody,
        key,
        serde_json::json!({"operation":"reapply_send_money", "history_token":token, "source":source}),
    )
}
async fn send_retained(
    world: &TransportProcessWorld,
    key: &str,
    amount: i64,
) -> (BankHttpCorrectionSource, u64) {
    let outcome = forwarded(node_mutation(&world.client, world.peer_address, &create(
        key, key, serde_json::json!({"operation":"send_money", "from":"fixture:101", "recipient":"fixture:1", "amount_minor_units":amount}),
    )).await);
    let (source, recovered, close_sequence) = finish(world, key, Some(outcome)).await;
    assert!(!recovered);
    (source, close_sequence)
}
async fn reverse(
    world: &TransportProcessWorld,
    key: &str,
    source: &BankHttpCorrectionSource,
) -> u64 {
    let outcome = forwarded(
        node_mutation(
            &world.client,
            world.peer_address,
            &create(
                key,
                key,
                serde_json::json!({"operation":"correct_send_money", "source":source}),
            ),
        )
        .await,
    );
    let (_, recovered, next) = finish(world, key, Some(outcome)).await;
    assert!(!recovered);
    next
}
async fn finish(
    world: &TransportProcessWorld,
    custody: &str,
    initial: Option<BankHttpMutationOutcome>,
) -> (BankHttpCorrectionSource, bool, u64) {
    let completed = match initial {
        Some(outcome @ BankHttpMutationOutcome::Completed { .. }) => outcome,
        Some(BankHttpMutationOutcome::Pending { .. }) | None => forwarded(node_mutation(&world.client, world.peer_address,
            &serde_json::json!({"command":"resume", "request_id":format!("finish-{custody}"), "custody_id":custody, "sequence":2, "controls":controls()})).await),
        other => panic!("native source owner: {other:?}"),
    };
    let BankHttpMutationOutcome::Completed {
        source,
        recovered,
        sequence,
        ..
    } = completed
    else {
        panic!("native retained delivery: {completed:?}");
    };
    (source, recovered, sequence + 1)
}
async fn choose(
    world: &TransportProcessWorld,
    source: &BankHttpCorrectionSource,
    id: &str,
) -> (String, Vec<BankHttpCorrectionSource>) {
    let first =
        serde_json::json!({"request_id":id, "source":source, "controls":history_controls()});
    let mut page = history(world, world.peer_address, "/v1/corrections/history", &first).await;
    assert_eq!(
        history(world, world.peer_address, "/v1/corrections/history", &first).await,
        page,
        "initial response replay"
    );
    let mut sources = Vec::new();
    let mut offset = 0;
    loop {
        let BankHttpCorrectionHistoryOutcome::Delivered {
            alternatives,
            continuation,
            complete,
            ..
        } = page
        else {
            panic!("native history page: {page:?}");
        };
        assert!(alternatives.len() <= 1);
        sources.extend(alternatives.into_iter().map(|row| row.source));
        if complete {
            return (continuation, sources);
        }
        offset += 1;
        let request = serde_json::json!({"request_id":format!("{id}-{offset}"), "source":source, "continuation":continuation, "controls":history_controls()});
        page = history(
            world,
            world.peer_address,
            "/v1/corrections/history/resume",
            &request,
        )
        .await;
        assert_eq!(
            history(
                world,
                world.peer_address,
                "/v1/corrections/history/resume",
                &request
            )
            .await,
            page,
            "last response replay"
        );
    }
}
fn history_controls() -> serde_json::Value {
    serde_json::json!({"deadline_milliseconds":5000, "maximum_history_work":128, "maximum_disclosure_results":16, "maximum_entries":1})
}
async fn history(
    world: &TransportProcessWorld,
    address: SocketAddr,
    path: &str,
    request: &serde_json::Value,
) -> BankHttpCorrectionHistoryOutcome {
    match post_node::<BankUserNodeCorrectionHistoryOutcome>(&world.client, address, path, request)
        .await
    {
        BankUserNodeCorrectionHistoryOutcome::Forwarded { response } => response,
        other => panic!("authenticated chooser forwarding: {other:?}"),
    }
}
async fn balances(world: &TransportProcessWorld, id: &str) -> (i64, i64) {
    (
        balance(
            world,
            world.peer_address,
            "fixture:101",
            &format!("{id}-source"),
        )
        .await,
        balance(
            world,
            world.primary_address,
            "fixture:100",
            &format!("{id}-recipient"),
        )
        .await,
    )
}
async fn assert_balances(
    world: &TransportProcessWorld,
    baseline: (i64, i64),
    amount: i64,
    id: &str,
) {
    assert_eq!(
        balances(world, id).await,
        (baseline.0 - amount, baseline.1 + amount)
    );
}
async fn activities(
    world: &TransportProcessWorld,
    id: &str,
) -> (
    Vec<BankHttpAccountActivityItem>,
    Vec<BankHttpAccountActivityItem>,
) {
    (
        activity(
            world,
            world.peer_address,
            "fixture:101",
            &format!("{id}-source"),
        )
        .await,
        activity(
            world,
            world.primary_address,
            "fixture:100",
            &format!("{id}-recipient"),
        )
        .await,
    )
}
fn assert_preserved(before: &[BankHttpAccountActivityItem], after: &[BankHttpAccountActivityItem]) {
    for row in before {
        assert_eq!(
            after.iter().find(|later| later.journal == row.journal),
            Some(row)
        );
    }
}
