//! One real process journey keeps HTTP selection separate from native lineage truth.
pub use super::*;
pub use bank_http_adapter::BankHttpCorrectionHistoryOutcome;
pub use bank_user_node::BankUserNodeCorrectionHistoryOutcome;

pub fn reapply(
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
pub async fn send_retained(
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
pub async fn reverse(
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
pub async fn finish(
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
pub async fn choose(
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
pub fn history_controls() -> serde_json::Value {
    serde_json::json!({"deadline_milliseconds":5000, "maximum_history_work":128, "maximum_disclosure_results":16, "maximum_entries":1})
}
pub async fn history(
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
pub async fn balances(world: &TransportProcessWorld, id: &str) -> (i64, i64) {
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
pub async fn assert_balances(
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
pub async fn activities(
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
pub fn assert_preserved(
    before: &[BankHttpAccountActivityItem],
    after: &[BankHttpAccountActivityItem],
) {
    for row in before {
        assert_eq!(
            after.iter().find(|later| later.journal == row.journal),
            Some(row)
        );
    }
}
