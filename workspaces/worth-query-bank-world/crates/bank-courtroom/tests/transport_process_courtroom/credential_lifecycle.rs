use std::time::{Duration, Instant};

use bank_http_adapter::{BankHttpAccountSummaryOutcome, BankHttpDenialKind};
use bank_user_node::{BankUserNodeAccountSummaryOutcome, BankUserNodeAuthorizationOutcome};

use super::live::{open_stream, read_until, revoke_session};
use super::world::TransportProcessWorld;

pub async fn assert_expired_session_fails_closed(world: &TransportProcessWorld) {
    assert_eq!(
        revoke_session(&world.client, world.primary_address).await,
        BankUserNodeAuthorizationOutcome::Revoked
    );
    // Leave enough lifetime for the real browser callback to finish under
    // full-workspace load while keeping expiry inside `read_until`'s bound.
    world.set_access_token_validity("seconds=5").await;
    world.authenticate_primary().await;

    let mut stream = open_stream(
        &world.client,
        world.primary_address,
        "process-expired-live",
        7_000,
    )
    .await;
    assert_eq!(stream.status(), reqwest::StatusCode::OK);
    let mut transcript = String::new();
    read_until(&mut stream, &mut transcript, "\"event\":\"opened\"").await;
    read_until(&mut stream, &mut transcript, "unauthenticated").await;
    drop(stream);

    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let outcome = summary(world).await;
        if matches!(
            outcome,
            BankUserNodeAccountSummaryOutcome::Forwarded {
                response: BankHttpAccountSummaryOutcome::Denied { denial, .. }
            } if denial.kind == BankHttpDenialKind::Unauthenticated
        ) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "short-lived process credential remained authoritative: {outcome:?}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn summary(world: &TransportProcessWorld) -> BankUserNodeAccountSummaryOutcome {
    world
        .client
        .post(format!(
            "http://{}/v1/queries/account-summary",
            world.primary_address
        ))
        .json(&serde_json::json!({
            "request_id": "process-expired-summary",
            "controls": {
                "deadline_milliseconds": 5_000,
                "maximum_results": 1,
                "maximum_work": 20_000
            },
            "account": "fixture:100"
        }))
        .send()
        .await
        .expect("expired-session query should respond")
        .json()
        .await
        .expect("expired-session response should remain typed")
}
