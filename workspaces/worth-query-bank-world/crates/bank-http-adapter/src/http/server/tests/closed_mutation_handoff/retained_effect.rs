//! Production HTTP close and abandoned handoff preserve original first-send custody.
use super::super::super::super::protocol::{
    BankHttpDenialKind, BankHttpEstateNotificationOutcome, BankHttpMutationOutcome as Outcome,
};
use super::super::{
    bind_application,
    fixture::application,
    mutation::{command, create, decode},
    BankHttpServerConfiguration,
};
use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction, EstateCaseId},
    model::{AccountId, BankPrincipalId},
    proposals::BankIdempotencyKey,
};
use bank_server::BankRecoveryIdempotencyResolution;
use std::sync::{Arc, Mutex};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    primary_graph::{
        WorthQueryCleanupDispatchDenial, WorthQueryCleanupDispatchPosture as Posture,
        WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
        WorthQueryExternalTransportOutcome,
    },
};

#[derive(Debug, Eq, PartialEq)]
struct ObservedDispatch {
    family: String,
    correlation: [u8; 32],
    effect: String,
    protocol: String,
    version: u32,
    maximum_payload_bytes: u64,
    payload: Vec<u8>,
}
#[derive(Default)]
struct LostReplyTransport(Mutex<Vec<ObservedDispatch>>);
impl WorthQueryExternalEffectTransport for LostReplyTransport {
    fn dispatch(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.lock().unwrap().push(ObservedDispatch {
            family: request.correlation_family().as_str().into(),
            correlation: *request.correlation_token(),
            effect: request.effect().into(),
            protocol: request.protocol_identity().as_str().into(),
            version: request.protocol_version().get(),
            maximum_payload_bytes: request.maximum_payload_bytes(),
            payload: request.payload().to_vec(),
        });
        WorthQueryExternalTransportOutcome::LostResponse
    }
}
#[tokio::test]
async fn abandoned_http_close_handoff_preserves_original_dispatch_and_recovery() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let transport = Arc::new(LostReplyTransport::default());
    application
        .runtime
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let scheduler = application
        .runtime
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let (listener, continuation) = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral()
            .with_opaque_handle_capacity(std::num::NonZeroUsize::new(1).unwrap()),
    )
    .await
    .unwrap()
    .into_parts();
    let server = listener.activate();
    let notified = reqwest::Client::new().post(format!("http://{}/v1/estate/notify-death", server.local_address()))
        .json(&serde_json::json!({"protocol":"v4", "request_id":"retained-first-send", "credential":super::super::credential_json(), "controls":{"deadline_milliseconds":5000}, "idempotency_key":"retained-original-key", "estate":"fixture:3", "notice":"fixture:12", "subject":"fixture:2"}))
        .send().await.unwrap().json::<BankHttpEstateNotificationOutcome>().await.unwrap();
    drop(scheduler);
    assert!(
        matches!(
            notified,
            BankHttpEstateNotificationOutcome::Owning {
                outcome: Outcome::Pending {
                    cleanup: Some(_),
                    ..
                }
            }
        ),
        "real cleanup failure retains its source: {notified:?}"
    );
    assert!(
        transport.0.lock().unwrap().is_empty(),
        "failed native cleanup cannot lose its first send by dispatching early"
    );
    assert!(
        matches!(
            continuation
                .execute(decode(command("retained-first-send", "close", 2)))
                .await,
            Outcome::Refused { .. }
        ),
        "indeterminate source cannot close before physical cleanup and keyed recovery"
    );
    assert!(matches!(
        continuation
            .execute(decode(command("retained-first-send", "release_cleanup", 3)))
            .await,
        Outcome::CollectionReleased {
            managed: Some(_),
            ..
        }
    ));
    assert!(matches!(
        continuation
            .execute(decode(command("retained-first-send", "resume_source", 4)))
            .await,
        Outcome::Recorded { .. }
    ));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = application.authenticate(&scope).await;
    let action = EstateAction::NotifyDeath {
        estate: EstateCaseId::new(3).unwrap(),
        notice: DeathNoticeId::new(12).unwrap(),
        subject: BankPrincipalId::new(2).unwrap(),
    };
    let key = BankIdempotencyKey::new("retained-original-key").unwrap();
    let BankRecoveryIdempotencyResolution::RecoveredCommitted(original) = application
        .runtime
        .resolve_estate_action_idempotency(&principal, action, &key, &scope)
        .unwrap()
    else {
        panic!("the actual owner resolves the original committed notification")
    };
    assert_eq!(
        application
            .death_notice_status(EstateCaseId::new(3).unwrap())
            .await,
        DeathNoticeStatus::NotificationRequested
    );
    let closed = continuation
        .execute(decode(command("retained-first-send", "close", 5)))
        .await;
    assert!(
        matches!(closed, Outcome::Recorded { .. }),
        "actual close outcome: {closed:?}"
    );
    assert!(
        matches!(continuation.execute(decode(create("before-handoff", 17))).await, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Saturated)
    );
    let abandoned = continuation
        .prepare_closed_mutation_handoff(decode(command("retained-first-send", "close", 6)))
        .await
        .unwrap_or_else(|cause| panic!("closed original prepares: {cause:?}"));
    drop(abandoned); // real public handoff has not been accepted; its exact record returns to worker
    assert!(
        matches!(continuation.execute(decode(create("after-lost-handoff", 17))).await, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Saturated)
    );
    let returned = continuation
        .prepare_closed_mutation_handoff(decode(command("retained-first-send", "close", 6)))
        .await
        .unwrap_or_else(|cause| {
            panic!("the same sequence receives its returned original: {cause:?}")
        })
        .receive();
    let (runtime, mut evidence) = returned.into_parts();
    let cleanup = evidence
        .released_collection_mut()
        .expect("actual collection stayed with its source")
        .managed_cleanup_receipt_mut()
        .expect("actual original cleanup custody survived transfer");
    assert_eq!(cleanup.dispatch_posture(), Posture::Pending);
    runtime
        .runtime()
        .resume_original_cleanup_dispatch(cleanup, &scope)
        .unwrap()
        .expect("original first-send result");
    assert_eq!(cleanup.dispatch_posture(), Posture::Observed);
    assert!(matches!(
        runtime
            .runtime()
            .resume_original_cleanup_dispatch(cleanup, &scope),
        Err(WorthQueryCleanupDispatchDenial::AlreadyAttempted)
    ));
    let outbox = original
        .dispatch_outbox()
        .expect("the original declared effect co-committed its bytes");
    assert_eq!(
        *transport.0.lock().unwrap(),
        vec![ObservedDispatch {
            family: outbox.correlation_family().into(),
            correlation: *outbox.correlation().bytes(),
            effect: outbox.effect().into(),
            protocol: outbox.protocol_identity().as_str().into(),
            version: outbox.protocol_version().get(),
            maximum_payload_bytes: outbox.maximum_payload_bytes(),
            payload: outbox.payload().to_vec(),
        }]
    );
    let _recovery = runtime
        .runtime()
        .open_cleanup_commit_recovery(cleanup)
        .expect("the actual lost-send original retains its recovery capability");
    assert!(matches!(
        continuation
            .execute(decode(create("after-confirmed-handoff", 17)))
            .await,
        Outcome::Completed {
            recovered: false,
            ..
        }
    ));
    assert!(matches!(
        continuation
            .execute(decode(command("after-confirmed-handoff", "close", 2)))
            .await,
        Outcome::Closed { .. }
    ));
    let BankRecoveryIdempotencyResolution::RecoveredCommitted(retry) = application
        .runtime
        .resolve_estate_action_idempotency(&principal, action, &key, &scope)
        .unwrap()
    else {
        panic!("later history still resolves the same original")
    };
    assert_eq!(retry, original);
    assert_eq!(transport.0.lock().unwrap().len(), 1);
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}
