//! A healthy native notification closes its original witness; the rail owner outlives the listener.
use super::super::super::protocol::{
    BankHttpEstateNotificationOutcome, BankHttpMutationOutcome as Outcome,
};
use super::{
    bind_application,
    fixture::application,
    mutation::{command, decode},
    BankHttpServerConfiguration,
};
use bank_domain::{
    estate::{DeathNoticeStatus, EstateCaseId},
    model::AccountId,
};
use std::sync::Arc;

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
