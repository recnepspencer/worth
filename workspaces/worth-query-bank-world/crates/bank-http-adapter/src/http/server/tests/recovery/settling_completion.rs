//! Safe retry of an effect whose completion is settling, then settled.
//!
//! A rail completion that is still publishing, or whose terminal index cannot
//! answer yet, asks the caller to try again later. Once the completion holds,
//! retry answers already-completed with the original commit and no refresh.

use super::super::super::super::protocol::{
    BankHttpCommitDescription, BankHttpDenial, BankHttpNextAction,
    BankHttpRecoverySafeRetryOutcome, BankHttpRecoveryStatus,
};
use super::*;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial,
};

struct CompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::SeqCst);
        WorthQueryExternalTransportOutcome::Completed
    }
}

/// The estate rail source, installed only after the synchronous completion,
/// so that completion's publication waits for it. No callback authenticates.
struct EstateRailSource;

impl WorthQueryInboundOccurrenceVerifier for EstateRailSource {
    fn audience(&self) -> &str {
        "bank-http-settling-completion"
    }

    fn source_identity(&self) -> &str {
        "rail-primary"
    }

    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        static IDENTITY: BoundaryProtocolIdentity =
            BoundaryProtocolIdentity::new("bank.estate.death-notification");
        &IDENTITY
    }

    fn protocol_version(&self) -> BoundaryProtocolVersion {
        BoundaryProtocolVersion::new(1)
    }

    fn verify(
        &self,
        _envelope: &[u8],
        _now_unix_seconds: u64,
        _maximum_work: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        Err(WorthQueryInboundVerificationDenial::AuthenticationFailed)
    }
}

async fn safe_retry(
    client: &reqwest::Client,
    address: SocketAddr,
    action: &RecoveryAction,
    request_id: &str,
    token: &str,
) -> BankHttpRecoverySafeRetryOutcome {
    post_typed::<BankHttpRecoverySafeRetryOutcome>(
        client,
        address,
        "/v1/recovery/safe-retry",
        &recovery_request(action, request_id, token),
    )
    .await
}

fn assert_try_again_later(outcome: &BankHttpRecoverySafeRetryOutcome, state: &str) {
    assert!(
        matches!(
            outcome,
            BankHttpRecoverySafeRetryOutcome::Denied {
                denial: BankHttpDenial {
                    kind: BankHttpDenialKind::Unavailable,
                    next_action: BankHttpNextAction::Retry,
                },
                ..
            }
        ),
        "{state} must ask the caller to try again later: {outcome:?}"
    );
}

fn assert_already_completed(
    outcome: BankHttpRecoverySafeRetryOutcome,
    original: BankHttpCommitDescription,
) {
    match outcome {
        BankHttpRecoverySafeRetryOutcome::AlreadyCompleted { commit, .. } => {
            assert_eq!(
                commit, original,
                "already-completed names the original commit"
            );
        }
        other => panic!("a settled completion must answer already-completed: {other:?}"),
    }
}

#[tokio::test]
async fn settling_completion_asks_to_retry_later_then_answers_already_completed() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let transport = Arc::new(CompletingTransport(AtomicUsize::new(0)));
    application
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("rail port should install once");
    let server = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .expect("HTTP server should bind");
    let client = reqwest::Client::new();
    let address = server.local_address();
    let action = recovery_action();
    let (original, token) =
        applied_notification(notify(&client, address, &action, "settling-commit").await);
    assert_eq!(transport.0.load(Ordering::SeqCst), 1);

    let pending = safe_retry(&client, address, &action, "publication-pending", &token).await;
    assert_try_again_later(&pending, "a pending completion publication");

    let route = application
        .runtime
        .install_estate_rail_completion_verifier(Arc::new(EstateRailSource))
        .expect("the estate rail source installs once");
    // The retained completion's World publication fails durably, leaving the
    // terminal index unable to answer until maintenance settles it.
    application.runtime.fail_next_durable_append_for_test();
    let failed = safe_retry(&client, address, &action, "publication-failed", &token).await;
    assert_try_again_later(&failed, "a failed completion publication");
    let unindexed = safe_retry(&client, address, &action, "index-unavailable", &token).await;
    assert_try_again_later(&unindexed, "an unavailable terminal index");

    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    application
        .runtime
        .maintain_estate_rail_completion(&route, &scope)
        .expect("maintenance resumes the unpublished completion");
    let completed = safe_retry(&client, address, &action, "completed", &token).await;
    assert_already_completed(completed, original);
    let replay = safe_retry(&client, address, &action, "completed-replay", &token).await;
    assert_already_completed(replay, original);
    assert_eq!(
        transport.0.load(Ordering::SeqCst),
        1,
        "no refused or settled retry contacted the rail"
    );

    let commit_replay = notify(&client, address, &action, "settled-commit-replay").await;
    assert!(
        matches!(
            commit_replay,
            BankHttpEstateNotificationOutcome::Applied {
                recovery_status: BankHttpRecoveryStatus::Completed,
                ..
            }
        ),
        "the commit replay reports the completed recovery: {commit_replay:?}"
    );
    server.shutdown().await.expect("server should shut down");
}
