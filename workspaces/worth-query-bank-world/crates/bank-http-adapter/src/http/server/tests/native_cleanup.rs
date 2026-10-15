//! Genuine receiver-owned foreign cleanup, original-thread retry, and actual SourceOnly adoption.
use super::super::super::protocol::{
    BankHttpDenialKind, BankHttpEstateNotificationOutcome, BankHttpMutationOutcome as Outcome,
};
use super::super::{
    native_route,
    recovery_executor::{self, AdmittedBankHttpNotificationRequest},
};
mod elevation;

use super::{
    bind_application,
    fixture::application,
    mutation::{command, decode},
    BankHttpServerConfiguration,
};
use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction, EstateCaseId},
    model::{AccountId, BankPrincipalId},
    proposals::BankIdempotencyKey,
};
use std::sync::Arc;
use tokio::sync::oneshot;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

#[tokio::test]
async fn notification_unpublished_cleanup_lost_reply_releases_on_original_thread_then_recovers() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let (listener, continuation) = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = listener.activate();
    let id = "native-notification";
    application.runtime.fail_next_durable_append_for_test();
    let _schedule = application
        .runtime
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let registry = Arc::clone(&continuation.transport.recovery);
    let (response, lost) = oneshot::channel();
    drop(lost);
    let (observation, observed) = oneshot::channel();
    let executing = Arc::clone(&application);
    continuation.submit_native(
        id.into(),
        native_route::job(
            move || async move {
                let returned = recovery_executor::notification::execute_notification(
                    executing.as_ref(),
                    registry,
                    request(id),
                    WorthQueryCancellationSource::new(),
                )
                .await;
                if let native_route::NativeRouteResult::Estate { outcome, .. } = &returned {
                    let _ = observation.send(outcome.clone());
                }
                returned
            },
            |_| panic!("reserved native notification must execute"),
            response,
        ),
    );
    let BankHttpEstateNotificationOutcome::Owning {
        outcome: Outcome::Pending {
            cleanup: Some(original),
            ..
        },
    } = observed.await.unwrap()
    else {
        panic!("actual unpublished source retains native cleanup failure")
    };
    assert!(original
        .failure
        .contains("SignalRuntimeThreadAffinityViolation"));
    assert_eq!(
        application
            .death_notice_status(EstateCaseId::new(3).unwrap())
            .await,
        DeathNoticeStatus::Reported
    );
    assert_eq!(
        continuation.quiesce().await.unwrap(),
        1,
        "lost reply does not release capacity"
    );
    let mut wrong = command(id, "release_cleanup", 2);
    wrong["credential"]["access_token"] = serde_json::json!("approver");
    assert!(
        matches!(continuation.execute(decode(wrong)).await, Outcome::Refused { denial, accepted_sequence: None, .. } if denial.kind == BankHttpDenialKind::PermissionDenied)
    );
    let source = WorthQueryCancellationSource::new();
    source.cancel();
    let admitted = super::super::mutation_command::admit(
        decode(command(id, "release_cleanup", 2)),
        std::time::Duration::from_secs(10),
    )
    .unwrap();
    let (response, refused) = oneshot::channel();
    let slot = Arc::clone(&continuation.transport.queue_slots)
        .try_acquire_owned()
        .unwrap();
    let worker =
        super::super::mutation_executor::routing_slot(id, continuation.transport.senders.len());
    continuation.transport.senders[worker]
        .send(super::super::mutation_executor::WorkerCommand::Execute {
            command: admitted,
            queued_slot: slot,
            cancellation: source,
            response,
        })
        .await
        .map_err(|_| ())
        .unwrap();
    assert!(
        matches!(refused.await.unwrap(), Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Cancelled)
    );
    let Outcome::CollectionReleased {
        managed: Some(released),
        ..
    } = continuation
        .execute(decode(command(id, "release_cleanup", 2)))
        .await
    else {
        panic!("fresh issuer returns real native physical receipt")
    };
    assert_eq!(released.run_identity, original.run_identity);
    assert!(released.resources_released);
    assert_eq!(
        continuation.quiesce().await.unwrap(),
        1,
        "physical release is not source completion"
    );
    assert!(matches!(
        continuation
            .execute(decode(command(id, "resume_source", 3)))
            .await,
        Outcome::Pending { .. }
    ));
    let Outcome::Completed {
        recovered: true,
        commit,
        ..
    } = continuation
        .execute(decode(command(id, "resume_source", 4)))
        .await
    else {
        panic!("original unpublished owner completes through genuine recovery")
    };
    assert_eq!(commit.emitted_effect_count, 1);
    assert_eq!(
        application
            .death_notice_status(EstateCaseId::new(3).unwrap())
            .await,
        DeathNoticeStatus::NotificationRequested
    );
    assert!(matches!(
        continuation.execute(decode(command(id, "close", 5))).await,
        Outcome::RailRecoveryReady { .. }
    ));
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}
fn request(id: &str) -> AdmittedBankHttpNotificationRequest {
    AdmittedBankHttpNotificationRequest {
        request_id: id.into(),
        credential: serde_json::from_value(super::credential_json()).unwrap(),
        idempotency_key: BankIdempotencyKey::new(format!("key-{id}")).unwrap(),
        action: EstateAction::NotifyDeath {
            estate: EstateCaseId::new(3).unwrap(),
            notice: DeathNoticeId::new(12).unwrap(),
            subject: BankPrincipalId::new(2).unwrap(),
        },
        deadline: std::time::Instant::now() + std::time::Duration::from_secs(5),
    }
}

#[tokio::test]
async fn healthy_notification_closes_original_witness_and_rail_owner_survives_listener_drop() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let (listener, continuation) = bind_application(
        Arc::clone(&application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = listener.activate();
    let client = reqwest::Client::new();
    let notified = client.post(format!("http://{}/v1/estate/notify-death", server.local_address()))
        .json(&serde_json::json!({"protocol":"v4", "request_id":"healthy-native-notification", "credential":super::credential_json(), "controls":{"deadline_milliseconds":5000}, "idempotency_key":"healthy-native-key", "estate":"fixture:3", "notice":"fixture:12", "subject":"fixture:2"}))
        .send().await.unwrap().json::<BankHttpEstateNotificationOutcome>().await.unwrap();
    assert!(matches!(
        notified,
        BankHttpEstateNotificationOutcome::Owning {
            outcome: Outcome::Completed {
                recovered: false,
                ..
            }
        }
    ));
    assert_eq!(
        application
            .death_notice_status(EstateCaseId::new(3).unwrap())
            .await,
        DeathNoticeStatus::NotificationRequested
    );
    let Outcome::RailRecoveryReady {
        recovery: Some(token),
        commit,
        ..
    } = continuation
        .execute(decode(command("healthy-native-notification", "close", 2)))
        .await
    else {
        panic!("actual closed witness supplies original external recovery")
    };
    assert_eq!(commit.emitted_effect_count, 1);
    drop(server);
    let inspection = continuation.inspect_recovery(serde_json::from_value(serde_json::json!({"protocol":"v4", "request_id":"after-listener", "credential":super::credential_json(), "controls":{"deadline_milliseconds":5000}, "recovery":token})).unwrap()).await;
    assert!(matches!(
        inspection,
        super::super::super::protocol::BankHttpRecoveryInspectionOutcome::Inspected { .. }
    ));
    assert!(continuation.finish().await.is_ok());
}
