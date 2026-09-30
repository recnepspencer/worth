//! Observe the retry's physical rail contact before releasing the callback.

use std::net::SocketAddr;
use std::time::Duration;

use bank_external_rail::inquire_dispatch_contact_count;
use bank_http_adapter::{
    BankHttpDenialKind, BankHttpRecoveryRetryDisposition, BankHttpRecoverySafeRetryOutcome,
};
use bank_user_node::BankUserNodeRecoverySafeRetryOutcome;

use super::super::post_node;

const TIMEOUT: Duration = Duration::from_secs(5);

pub(super) async fn count(address: SocketAddr) -> u64 {
    inquire_dispatch_contact_count(address, TIMEOUT)
        .await
        .expect("rail contact count should remain readable")
}

pub(super) async fn await_next(address: SocketAddr, previous: u64) {
    tokio::time::timeout(TIMEOUT, async {
        loop {
            let observed = count(address).await;
            assert!(observed <= previous + 1, "only one retry is in flight");
            if observed == previous + 1 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("owner retry must reach the rail while callback remains held");
}

pub(super) fn assert_racing_retry_result(result: &BankUserNodeRecoverySafeRetryOutcome) {
    assert!(
        matches!(
            result,
            BankUserNodeRecoverySafeRetryOutcome::Forwarded {
                response: BankHttpRecoverySafeRetryOutcome::Applied { .. }
                    | BankHttpRecoverySafeRetryOutcome::Denied {
                        denial: bank_http_adapter::BankHttpDenial {
                            kind: BankHttpDenialKind::Stale,
                            ..
                        },
                        ..
                    }
            }
        ),
        "racing retry must execute or find callback-owned terminal: {result:?}"
    );
}

pub(super) async fn assert_post_terminal_retry(
    client: &reqwest::Client,
    node_address: SocketAddr,
    recovery: &str,
    rail_address: SocketAddr,
    expected_contacts: u64,
) {
    let result = post_node::<BankUserNodeRecoverySafeRetryOutcome>(
        client,
        node_address,
        "/v1/recovery/safe-retry",
        &serde_json::json!({
            "request_id": "process-rail-callback-after-terminal",
            "controls": { "deadline_milliseconds": 5_000 },
            "recovery": recovery
        }),
    )
    .await;
    assert!(
        matches!(
            &result,
            BankUserNodeRecoverySafeRetryOutcome::Forwarded {
                response: BankHttpRecoverySafeRetryOutcome::Denied {
                    denial: bank_http_adapter::BankHttpDenial {
                        kind: BankHttpDenialKind::Stale,
                        ..
                    },
                    ..
                } | BankHttpRecoverySafeRetryOutcome::Applied {
                    disposition: BankHttpRecoveryRetryDisposition::AlreadyRetried,
                    ..
                }
            }
        ),
        "terminal callback prevents another fresh safe retry: {result:?}"
    );
    assert_eq!(
        count(rail_address).await,
        expected_contacts,
        "post-terminal retry cannot contact the rail again"
    );
}
