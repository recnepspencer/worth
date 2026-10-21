//! Transport concurrency, cancellation and listener-error custody use the real worker entrance.
use super::super::super::protocol::{BankHttpDenialKind, BankHttpMutationOutcome as Outcome};
use super::fixture::{application, held_authentication_application};
use super::mutation::{command, create, decode};
use super::{bind_application, BankHttpServerConfiguration};
use bank_domain::model::AccountId;
use std::sync::Arc;

#[tokio::test]
async fn owning_concurrent_sequence_reads_once_without_protected_row_replay() {
    let (server, continuation) = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    assert!(matches!(
        continuation.execute(decode(create("concurrent", 17))).await,
        Outcome::Completed { .. }
    ));
    let (first, second) = tokio::join!(
        continuation.execute(decode(command("concurrent", "read", 2))),
        continuation.execute(decode(command("concurrent", "read", 2)))
    );
    assert!(matches!(
        (&first, &second),
        (
            Outcome::Delivered { .. },
            Outcome::Refused {
                accepted_sequence: Some(2),
                ..
            }
        ) | (
            Outcome::Refused {
                accepted_sequence: Some(2),
                ..
            },
            Outcome::Delivered { .. }
        )
    ));
    assert!(matches!(
        continuation
            .execute(decode(command("concurrent", "close", 3)))
            .await,
        Outcome::Closed { .. }
    ));
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}
#[tokio::test]
async fn owning_queued_timeout_is_unknown_and_cancelled_command_preserves_custody() {
    let (application, hold) = held_authentication_application(AccountId::new(100).unwrap());
    let (server, continuation) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    hold.release(1);
    assert!(matches!(
        continuation.execute(decode(create("deadline", 17))).await,
        Outcome::Completed { .. }
    ));
    let mut close = command("deadline", "close", 2);
    close["controls"]["deadline_milliseconds"] = serde_json::json!(1);
    assert!(matches!(
        continuation.execute(decode(close)).await,
        Outcome::OutcomeUnknown { .. }
    ));
    hold.wait_for_calls(2).await;
    hold.release(2); // cancelled close authentication, then a genuinely fresh close
    assert!(matches!(
        continuation
            .execute(decode(command("deadline", "close", 2)))
            .await,
        Outcome::Closed { .. }
    ));
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}
#[tokio::test]
async fn owning_shutdown_failure_drop_preserves_independent_resumption() {
    let (server, continuation) = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    assert!(matches!(
        continuation.execute(decode(create("error-drop", 17))).await,
        Outcome::Completed { .. }
    ));
    let failure = server
        .shutdown()
        .await
        .err()
        .expect("native custody remains");
    drop(failure);
    assert!(
        matches!(continuation.execute(decode(command("error-drop","read",2))).await, Outcome::Delivered { activity, .. } if activity.entries.len() == 3)
    );
    assert!(matches!(
        continuation
            .execute(decode(command("error-drop", "close", 3)))
            .await,
        Outcome::Closed { .. }
    ));
    assert!(continuation.finish().await.is_ok());
}
#[tokio::test]
async fn owning_cancelled_wait_does_not_close_native_custody() {
    let (application, hold) = held_authentication_application(AccountId::new(100).unwrap());
    let (server, continuation) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    hold.release(1);
    assert!(matches!(
        continuation.execute(decode(create("cancel", 17))).await,
        Outcome::Completed { .. }
    ));
    let caller = continuation.clone();
    let waiting =
        tokio::spawn(async move { caller.execute(decode(command("cancel", "close", 2))).await });
    hold.wait_for_calls(2).await;
    waiting.abort();
    assert!(waiting.await.unwrap_err().is_cancelled());
    hold.release(2);
    assert!(
        matches!(continuation.execute(decode(command("cancel","read",2))).await, Outcome::Delivered { activity, .. } if activity.entries.len() == 3)
    );
    hold.release(1);
    assert!(matches!(
        continuation
            .execute(decode(command("cancel", "close", 3)))
            .await,
        Outcome::Closed { .. }
    ));
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}
#[tokio::test]
async fn owning_current_wire_rejects_cross_command_inputs_before_effects() {
    let (server, continuation) = bind_application(
        Arc::new(application(AccountId::new(100).unwrap())),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = server.activate();
    let client = reqwest::Client::new();
    for mut specimen in [
        command("absent", "resume", 1),
        command("absent", "read", 1),
        command("absent", "close", 1),
    ] {
        specimen["operation"] = create("substitute", 99)["operation"].clone();
        let outcome = client
            .post(format!("http://{}/v1/mutations", server.local_address()))
            .json(&specimen)
            .send()
            .await
            .unwrap()
            .json::<Outcome>()
            .await
            .unwrap();
        assert!(
            matches!(outcome, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::MalformedRequest)
        );
    }
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}

#[tokio::test]
async fn owning_global_command_capacity_refuses_before_second_authentication() {
    let (application, hold) = held_authentication_application(AccountId::new(100).unwrap());
    let (prepared, continuation) = bind_application(
        Arc::new(application),
        BankHttpServerConfiguration::local_ephemeral()
            .with_queue_capacity(std::num::NonZeroUsize::new(1).unwrap())
            .with_maximum_concurrency(std::num::NonZeroUsize::new(2).unwrap()),
    )
    .await
    .unwrap()
    .into_parts();
    let server = prepared.activate();
    let first = continuation.clone();
    let waiting = tokio::spawn(async move { first.execute(decode(create("worker-a", 17))).await });
    hold.wait_for_calls(1).await;
    assert!(
        matches!(continuation.execute(decode(create("worker-b",99))).await, Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Saturated)
    );
    assert_eq!(hold.call_count(), 1);
    hold.release(1);
    assert!(matches!(waiting.await.unwrap(), Outcome::Completed { .. }));
    hold.release(1);
    assert!(matches!(
        continuation
            .execute(decode(command("worker-a", "close", 2)))
            .await,
        Outcome::Closed { .. }
    ));
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}

#[tokio::test]
async fn closed_money_worker_is_unavailable_with_exact_runtime_cleanup_access() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let (prepared, continuation) = bind_application(
        application.clone(),
        BankHttpServerConfiguration::local_ephemeral(),
    )
    .await
    .unwrap()
    .into_parts();
    let server = prepared.activate();
    let retained = continuation.clone();
    assert!(continuation.finish().await.is_ok());
    assert!(std::ptr::eq(retained.runtime(), &application.runtime));
    assert!(
        matches!(retained.execute(decode(create("closed-worker", 17))).await,
        Outcome::Rejected { denial, .. } if denial.kind == BankHttpDenialKind::Unavailable)
    );
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn unauthorized_initial_custody_closes_lawfully_and_returns_capacity() {
    use worth_query_host::facade::admission::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let account = AccountId::new(100).unwrap();
    let application = Arc::new(application(account));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let reader = application.authenticate(&scope).await;
    let entries = || {
        application
            .runtime
            .account_activity(account)
            .as_principal(&reader)
            .execute(bank_server::BankReadControls::current(scope.clone(), 16).unwrap())
            .unwrap()
            .rows()[0]
            .entries()
            .len()
    };
    let before = entries();
    let (prepared, continuation) = bind_application(
        application.clone(),
        BankHttpServerConfiguration::local_ephemeral()
            .with_opaque_handle_capacity(std::num::NonZeroUsize::MIN),
    )
    .await
    .unwrap()
    .into_parts();
    let server = prepared.activate();
    let mut denied = create("unexecuted", 17);
    denied["credential"]["access_token"] = serde_json::json!("estate-subject");
    assert!(matches!(continuation.execute(decode(denied)).await,
        Outcome::Refused { denial, accepted_sequence: Some(1), .. } if denial.kind == BankHttpDenialKind::PermissionDenied));
    assert_eq!(
        entries(),
        before,
        "ordinary authorization refusal has no source effect"
    );
    let mut close = command("unexecuted", "close", 2);
    close["credential"]["access_token"] = serde_json::json!("estate-subject");
    assert!(matches!(
        continuation.execute(decode(close)).await,
        Outcome::Closed { .. }
    ));
    assert!(matches!(
        continuation
            .execute(decode(create("lawful-after-refusal", 17)))
            .await,
        Outcome::Completed {
            recovered: false,
            ..
        }
    ));
    assert_eq!(entries(), before + 1);
    assert!(matches!(
        continuation
            .execute(decode(command("lawful-after-refusal", "close", 2)))
            .await,
        Outcome::Closed { .. }
    ));
    server.shutdown().await.unwrap();
    assert!(continuation.finish().await.is_ok());
}
