use super::super::super::super::protocol::{
    BankHttpRecoveryRetryDisposition, BankHttpRecoverySafeRetryOutcome, BankHttpRecoveryStatus,
};
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
};

struct CompletingTransport;

impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[tokio::test]
async fn terminal_retry_replay_survives_until_eviction_then_commit_replay_reports_omitted_token() {
    let application = application(AccountId::new(100).unwrap());
    application
        .runtime
        .install_external_effect_transport(Arc::new(CompletingTransport))
        .expect("rail port should install once");
    let server = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral()
            .with_opaque_handle_capacity(NonZeroUsize::new(1).unwrap())
            .with_opaque_handle_lifetime(Duration::ZERO),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let first_action = recovery_action();
    let first = notify_with_key(
        &client,
        server.local_address(),
        &first_action,
        "first-notification",
        "first-key",
    )
    .await;
    let (_, token) = applied_notification(first);
    let retry = post_typed::<BankHttpRecoverySafeRetryOutcome>(
        &client,
        server.local_address(),
        "/v1/recovery/safe-retry",
        &recovery_request(&first_action, "first-retry", &token),
    )
    .await;
    assert!(matches!(
        retry,
        BankHttpRecoverySafeRetryOutcome::Applied {
            disposition: BankHttpRecoveryRetryDisposition::Retried,
            external_completion: true,
            ..
        }
    ));
    let retry_replay = post_typed::<BankHttpRecoverySafeRetryOutcome>(
        &client,
        server.local_address(),
        "/v1/recovery/safe-retry",
        &recovery_request(&first_action, "first-retry-replay", &token),
    )
    .await;
    assert!(matches!(
        retry_replay,
        BankHttpRecoverySafeRetryOutcome::Applied {
            disposition: BankHttpRecoveryRetryDisposition::AlreadyRetried,
            ..
        }
    ));
    let completed_commit_replay = notify_with_key(
        &client,
        server.local_address(),
        &first_action,
        "first-completed-replay",
        "first-key",
    )
    .await;
    assert!(matches!(
        completed_commit_replay,
        BankHttpEstateNotificationOutcome::Applied {
            recovery_status: BankHttpRecoveryStatus::Completed,
            recovery: Some(_),
            ..
        }
    ));
    let alternate = alternate_recovery_action();
    applied_notification(
        notify_with_key(
            &client,
            server.local_address(),
            &alternate,
            "second-notification",
            "second-key",
        )
        .await,
    );
    let old_replay = notify_with_key(
        &client,
        server.local_address(),
        &first_action,
        "first-after-eviction",
        "first-key",
    )
    .await;
    assert!(
        matches!(
            &old_replay,
            BankHttpEstateNotificationOutcome::Applied {
                disposition:
                    super::super::super::super::protocol::BankHttpCommitDisposition::AlreadyCommitted,
                recovery: None,
                recovery_status: BankHttpRecoveryStatus::Completed,
                ..
            }
        ),
        "unexpected post-eviction replay: {old_replay:?}"
    );
    server.shutdown().await.expect("server should shut down");
}
