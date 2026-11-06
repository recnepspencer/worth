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
    fn admission_clock(
        &self,
    ) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock
    {
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()
    }

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
    let (server, _mutation_continuation) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind")
    .into_parts();
    let server = server.activate();
    let client = reqwest::Client::new();
    let action = recovery_action();
    let (_, token) = rail_notification_parts(
        close_notification(
            &client,
            server.local_address(),
            notify(
                &client,
                server.local_address(),
                &action,
                "unresolved-commit",
            )
            .await,
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
    let replay = close_notification(&client, server.local_address(), replay).await;
    assert!(matches!(
        replay,
        crate::http::protocol::BankHttpMutationOutcome::RecordedRailRecoveryReady {
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
    assert!(_mutation_continuation.finish().await.is_ok());
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
    let (server, _mutation_continuation) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral().with_opaque_handle_lifetime(Duration::ZERO),
    )
    .await
    .expect("HTTP server should bind")
    .into_parts();
    let server = server.activate();
    let client = reqwest::Client::new();
    let action = recovery_action();
    let (_, token) = rail_notification_parts(
        close_notification(
            &client,
            server.local_address(),
            notify(&client, server.local_address(), &action, "expiring-commit").await,
        )
        .await,
    );
    time.advance_to(5_601);
    let replay = notify(&client, server.local_address(), &action, "expired-replay").await;
    let replay = close_notification(&client, server.local_address(), replay).await;
    assert!(matches!(
        replay,
        crate::http::protocol::BankHttpMutationOutcome::RecordedRailRecoveryReady {
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
    assert!(_mutation_continuation.finish().await.is_ok());
}

#[tokio::test]
async fn an_expired_capability_cannot_disclose_a_retained_retry_token_on_close() {
    let time = ControlledTime::at(2_000);
    let application =
        super::super::fixture::expiring_notification::application_with_expiring_notification(
            AccountId::new(100).unwrap(),
            time.clone(),
            2_100,
        );
    let (server, mutations) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    let client = reqwest::Client::new();
    let address = server.local_address();
    let action = recovery_action();
    let original = notify(&client, address, &action, "original-before-expiry").await;
    let (_, token) = rail_notification_parts(close_notification(&client, address, original).await);
    assert!(!token.is_empty());
    let retry = notify(&client, address, &action, "retry-before-expiry").await;
    assert!(matches!(
        &retry,
        BankHttpEstateNotificationOutcome::Owning {
            outcome: crate::http::protocol::BankHttpMutationOutcome::Recorded { .. }
        }
    ));
    time.advance_to(2_101);
    let refused = close_notification(&client, address, retry).await;
    assert!(
        matches!(refused, crate::http::protocol::BankHttpMutationOutcome::Refused { denial, .. }
        if denial.kind == BankHttpDenialKind::PermissionDenied),
        "expiry must deny token and receipt disclosure: {refused:?}"
    );
    server.shutdown().await.unwrap();
    assert!(mutations.finish().await.is_ok());
}
