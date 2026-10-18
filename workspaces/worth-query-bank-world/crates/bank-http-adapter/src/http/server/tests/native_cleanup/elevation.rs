//! Each genuine elevation failure transfers its native owner, not a reconstructed phase token.
use super::super::super::super::protocol::*;
use super::super::{
    bind_application,
    fixture::application,
    mutation::{command, decode},
    BankHttpServerConfiguration,
};
use bank_domain::model::AccountId;
use std::sync::Arc;

#[tokio::test]
async fn all_four_elevation_transitions_keep_actual_cleanup_and_business_evidence() {
    for target in 0..4 {
        let application = Arc::new(application(AccountId::new(100).unwrap()));
        let (listener, continuation) = bind_application(
            Arc::clone(&application),
            BankHttpServerConfiguration::local_ephemeral(),
        )
        .await
        .unwrap()
        .into_parts();
        let server = listener.activate();
        let origin = format!("http://{}", server.local_address());
        let client = reqwest::Client::new();
        let mut token = None;
        for transition in 0..=target {
            let id = format!("native-elevation-{target}-{transition}");
            let schedule = (transition == target).then(|| {
                application.runtime.fail_next_durable_append_for_test();
                application
                    .runtime
                    .schedule_next_managed_cleanup_on_worker_for_test()
                    .unwrap()
            });
            let (route, value) = request(transition, &id, token.as_deref());
            let response = client
                .post(format!("{origin}{route}"))
                .json(&value)
                .send()
                .await
                .unwrap();
            if transition < target {
                token = Some(match transition {
                    0 => match response
                        .json::<BankHttpElevationRequestOutcome>()
                        .await
                        .unwrap()
                    {
                        BankHttpElevationRequestOutcome::Requested { elevation, .. } => elevation,
                        other => panic!("healthy native request: {other:?}"),
                    },
                    1 => match response
                        .json::<BankHttpElevationApprovalOutcome>()
                        .await
                        .unwrap()
                    {
                        BankHttpElevationApprovalOutcome::Approved { elevation, .. } => elevation,
                        other => panic!("healthy native approval: {other:?}"),
                    },
                    2 => match response
                        .json::<BankHttpElevationRevocationOutcome>()
                        .await
                        .unwrap()
                    {
                        BankHttpElevationRevocationOutcome::Closed {
                            mandatory_review, ..
                        } => mandatory_review,
                        other => panic!("healthy native revocation: {other:?}"),
                    },
                    _ => unreachable!(),
                });
                continue;
            }
            let custody = match transition {
                0 => match response
                    .json::<BankHttpElevationRequestOutcome>()
                    .await
                    .unwrap()
                {
                    BankHttpElevationRequestOutcome::CleanupRequired {
                        custody_id,
                        sequence: 1,
                        ..
                    } => custody_id,
                    other => panic!("actual request cleanup owner: {other:?}"),
                },
                1 => match response
                    .json::<BankHttpElevationApprovalOutcome>()
                    .await
                    .unwrap()
                {
                    BankHttpElevationApprovalOutcome::CleanupRequired {
                        custody_id,
                        sequence: 1,
                        ..
                    } => custody_id,
                    other => panic!("actual approval cleanup owner: {other:?}"),
                },
                2 => match response
                    .json::<BankHttpElevationRevocationOutcome>()
                    .await
                    .unwrap()
                {
                    BankHttpElevationRevocationOutcome::CleanupRequired {
                        custody_id,
                        sequence: 1,
                        ..
                    } => custody_id,
                    other => panic!("actual revocation cleanup owner: {other:?}"),
                },
                3 => match response
                    .json::<BankHttpMandatoryReviewOutcome>()
                    .await
                    .unwrap()
                {
                    BankHttpMandatoryReviewOutcome::CleanupRequired {
                        custody_id,
                        sequence: 1,
                        ..
                    } => custody_id,
                    other => panic!("actual review cleanup owner: {other:?}"),
                },
                _ => unreachable!(),
            };
            drop(schedule);
            assert_eq!(continuation.quiesce().await.unwrap(), 1);
            let mut release = command(&custody, "release_cleanup", 2);
            release["credential"] = value["credential"].clone();
            assert!(matches!(
                continuation.execute(decode(release)).await,
                BankHttpMutationOutcome::CleanupReleased {
                    resources_released: true,
                    ..
                }
            ));
            assert_eq!(
                continuation.quiesce().await.unwrap(),
                1,
                "physical receipt cannot settle business"
            );
            let mut handoff = command(&custody, "close", 3);
            handoff["credential"] = value["credential"].clone();
            if target == 0 {
                let admitted = super::super::super::mutation_command::admit(
                    decode(handoff.clone()),
                    std::time::Duration::from_secs(10),
                )
                .unwrap();
                let index = super::super::super::mutation_executor::routing_slot(
                    &custody,
                    continuation.transport.senders.len(),
                );
                let (response, receiver) = tokio::sync::oneshot::channel();
                let queued_slot = Arc::clone(&continuation.transport.queue_slots)
                    .try_acquire_owned()
                    .unwrap();
                continuation.transport.senders[index].send(super::super::super::mutation_executor::WorkerCommand::Handoff {
                    return_sender: continuation.transport.senders[index].clone(), command: admitted, queued_slot,
                    cancellation: worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new(), response,
                }).await.map_err(|_| ()).unwrap();
                let delivery = receiver
                    .await
                    .unwrap()
                    .unwrap_or_else(|cause| panic!("actual delivery: {cause:?}"));
                assert_eq!(
                    continuation.quiesce().await.unwrap(),
                    1,
                    "unacknowledged host receipt still blocks shutdown"
                );
                drop(delivery); // same unique native evidence returns through its reserved slot
                assert_eq!(continuation.quiesce().await.unwrap(), 1);
            }
            let handoff = continuation
                .take_released_cleanup(decode(handoff))
                .await
                .unwrap_or_else(|cause| panic!("actual host evidence: {cause:?}"));
            assert!(std::ptr::eq(handoff.runtime(), &application.runtime));
            assert!(handoff
                .evidence()
                .receipt()
                .inspection()
                .resources_released());
            assert!(matches!(handoff.evidence().business(), worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(_)));
            assert_eq!(
                continuation.quiesce().await.unwrap(),
                0,
                "actual host now owns original uncommitted evidence"
            );
            // Handoff deliberately remains reachable while the listener shuts down. No business success is claimed.
            server.shutdown().await.unwrap();
            assert!(continuation.finish().await.is_ok());
            drop(handoff);
            break;
        }
    }
}
fn request(transition: usize, id: &str, token: Option<&str>) -> (&'static str, serde_json::Value) {
    let actor = match transition {
        0 => "test-only",
        1 | 2 => "approver",
        3 => "reviewer",
        _ => unreachable!(),
    };
    let mut value = serde_json::json!({"protocol":"v4", "request_id":id, "credential": {"id_token":actor,"access_token":actor,"nonce":"test-only"}, "controls":{"deadline_milliseconds":5000}, "idempotency_key":format!("key-{id}")});
    let route = match transition {
        0 => {
            for (key, data) in serde_json::json!({"estate":"fixture:3", "access":31,"mandatory_review":32,"upper_bound_grant":20,"reason":"prevent_immediate_loss","field":"account_details","duration_seconds":300}).as_object().unwrap() { value[key] = data.clone(); }
            "/v1/estate/elevation/request"
        }
        1 | 2 => {
            value["elevation"] = serde_json::json!(token.unwrap());
            if transition == 1 {
                "/v1/estate/elevation/approve"
            } else {
                "/v1/estate/elevation/revoke"
            }
        }
        3 => {
            value["mandatory_review"] = serde_json::json!(token.unwrap());
            "/v1/estate/elevation/review"
        }
        _ => unreachable!(),
    };
    (route, value)
}
