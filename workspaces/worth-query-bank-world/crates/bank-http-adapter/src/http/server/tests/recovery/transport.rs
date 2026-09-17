use super::super::super::super::protocol::{
    BankHttpRecoveryRetryDisposition, BankHttpRecoverySafeRetryOutcome, BankHttpRecoveryStatus,
};
use super::super::fixture::application_with_authorization_time;
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome, WorthQueryRuntimeTimeSource,
    WorthQueryRuntimeTimeSourceDenial,
};

#[derive(Clone)]
struct ControlledTime(Arc<std::sync::atomic::AtomicU64>);

impl ControlledTime {
    fn at(seconds: u64) -> Self {
        Self(Arc::new(std::sync::atomic::AtomicU64::new(seconds)))
    }

    fn advance_to(&self, seconds: u64) {
        self.0.store(seconds, Ordering::SeqCst);
    }
}

impl WorthQueryRuntimeTimeSource for ControlledTime {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(UNIX_EPOCH + Duration::from_secs(self.0.load(Ordering::SeqCst)))
    }
}

struct TwiceUnresolvedTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for TwiceUnresolvedTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        if self.0.fetch_add(1, Ordering::SeqCst) < 2 {
            WorthQueryExternalTransportOutcome::TimedOut
        } else {
            WorthQueryExternalTransportOutcome::Completed
        }
    }
}

#[tokio::test]
async fn unresolved_http_retry_keeps_the_same_token_until_completion() {
    let application = application(AccountId::new(100).unwrap());
    let transport = Arc::new(TwiceUnresolvedTransport(AtomicUsize::new(0)));
    application
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("rail port should install once");
    let server = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let action = recovery_action();
    let (_, token) = applied_notification(
        notify(
            &client,
            server.local_address(),
            &action,
            "unresolved-commit",
        )
        .await,
    );
    let first = post_typed::<BankHttpRecoverySafeRetryOutcome>(
        &client,
        server.local_address(),
        "/v1/recovery/safe-retry",
        &recovery_request(&action, "unresolved-retry", &token),
    )
    .await;
    assert!(matches!(
        first,
        BankHttpRecoverySafeRetryOutcome::Denied { denial, .. }
            if denial.kind == BankHttpDenialKind::Unavailable
    ));
    let replay = notify(
        &client,
        server.local_address(),
        &action,
        "unresolved-replay",
    )
    .await;
    assert!(matches!(
        replay,
        BankHttpEstateNotificationOutcome::Applied {
            recovery: Some(ref replay_token),
            recovery_status: BankHttpRecoveryStatus::TokenIssued,
            ..
        } if replay_token == &token
    ));
    let second = post_typed::<BankHttpRecoverySafeRetryOutcome>(
        &client,
        server.local_address(),
        "/v1/recovery/safe-retry",
        &recovery_request(&action, "completed-retry", &token),
    )
    .await;
    assert!(matches!(
        second,
        BankHttpRecoverySafeRetryOutcome::Applied {
            disposition: BankHttpRecoveryRetryDisposition::Retried,
            external_completion: true,
            ..
        }
    ));
    assert_eq!(transport.0.load(Ordering::SeqCst), 3);
    server.shutdown().await.expect("server should shut down");
}

#[tokio::test]
async fn expired_query_recovery_replay_reports_operator_required() {
    let time = ControlledTime::at(2_000);
    let application =
        application_with_authorization_time(AccountId::new(100).unwrap(), time.clone());
    application
        .runtime
        .install_external_effect_transport(Arc::new(TwiceUnresolvedTransport(AtomicUsize::new(0))))
        .expect("rail port should install once");
    let server = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral().with_opaque_handle_lifetime(Duration::ZERO),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let action = recovery_action();
    let (_, token) = applied_notification(
        notify(&client, server.local_address(), &action, "expiring-commit").await,
    );
    time.advance_to(5_601);
    let replay = notify(&client, server.local_address(), &action, "expired-replay").await;
    assert!(matches!(
        replay,
        BankHttpEstateNotificationOutcome::Applied {
            disposition: crate::http::protocol::BankHttpCommitDisposition::AlreadyCommitted,
            recovery: None,
            recovery_status: BankHttpRecoveryStatus::OperatorRequired,
            ..
        }
    ));
    let old_token = inspect_recovery(&client, server.local_address(), &action, &token).await;
    assert!(matches!(
        old_token,
        BankHttpRecoveryInspectionOutcome::Denied { .. }
    ));
    server.shutdown().await.expect("server should shut down");
}
