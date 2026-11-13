//! Production HTTP close and abandoned handoff preserve original first-send custody.
use super::super::super::super::protocol::{
    BankHttpDenialKind, BankHttpEstateNotificationOutcome, BankHttpMutationOutcome as Outcome,
};
use super::super::super::{
    cleanup_description,
    closed_mutation_custody::{discharged, may_release},
    mutation_application::{parse_deposit, AdmittedBankHttpMutation},
    mutation_construction::start,
    mutation_custody::{CustodyDescription, SourceMutationCustody},
};
use super::super::{
    bind_application,
    fixture::{application, CausalHttpApplication},
    mutation::{command, create, decode},
    BankHttpServerConfiguration,
};
use bank_domain::{
    estate::{DeathNoticeId, DeathNoticeStatus, EstateAction, EstateCaseId},
    model::{AccountId, BankPrincipalId},
    proposals::BankIdempotencyKey,
};
use bank_server::{BankMutationControls, BankRecoveryIdempotencyResolution};
use std::sync::{Arc, Mutex};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_entry::{
        WorthQueryOperationCleanupReceipt as CleanupReceipt, WorthQuerySourceOnlyPosture,
    },
    primary_graph::{
        WorthQueryApplicationManagedCleanupReceipt as ManagedReceipt,
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
    let next = |kind, sequence| decode(command("retained-first-send", kind, sequence));
    let notified = reqwest::Client::new().post(format!("http://{}/v1/estate/notify-death", server.local_address()))
        .json(&serde_json::json!({"protocol":"v4", "request_id":"retained-first-send", "credential":super::super::credential_json(), "controls":{"deadline_milliseconds":5000}, "idempotency_key":"retained-original-key", "estate":"fixture:3", "notice":"fixture:12", "subject":"fixture:2"}))
        .send().await.unwrap().json::<BankHttpEstateNotificationOutcome>().await.unwrap();
    drop(scheduler);
    let BankHttpEstateNotificationOutcome::Owning {
        outcome: Outcome::Pending { cleanup, .. },
    } = &notified
    else {
        panic!("actual pending original: {notified:?}")
    };
    assert_eq!(cleanup.len(), 1, "one authored notification failure");
    let [failure] = cleanup.as_slice() else {
        panic!("one notification failure")
    };
    assert!(!failure.run_identity.is_empty());
    assert!(!failure.graph_session_identity.is_empty());
    assert!(!failure.failure.is_empty());
    let pending_wire = serde_json::to_value(&notified).unwrap();
    assert_eq!(
        serde_json::from_value::<BankHttpEstateNotificationOutcome>(pending_wire).unwrap(),
        notified
    );
    assert!(
        transport.0.lock().unwrap().is_empty(),
        "failed native cleanup cannot lose its first send by dispatching early"
    );
    assert!(
        matches!(
            continuation.execute(next("close", 2)).await,
            Outcome::Refused { .. }
        ),
        "indeterminate source cannot close before physical cleanup and keyed recovery"
    );
    let released = continuation.execute(next("release_cleanup", 3)).await;
    let Outcome::CollectionReleased { managed, .. } = &released else {
        panic!("actual released collection: {released:?}")
    };
    assert_eq!(managed.len(), 1, "one original native cleanup receipt");
    assert!(matches!(
        continuation.execute(next("resume_source", 4)).await,
        Outcome::Recorded { .. }
    ));
    let scope = request_scope();
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
    let closed = continuation.execute(next("close", 5)).await;
    assert!(matches!(closed, Outcome::Recorded { .. }), "{closed:?}");
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
        .unwrap_or_else(|cause| panic!("returned original: {cause:?}"))
        .receive();
    let (runtime, mut evidence) = returned.into_parts();
    let (_neutral_owner, neutral, _) = deposit_collection(&application, "negative-neutral").await;
    let collection = evidence
        .released_collection_mut()
        .expect("actual collection stayed with its source");
    assert_eq!(collection.managed_cleanup_receipts().len(), 1);
    let [cleanup] = collection.managed_cleanup_receipts_mut() else {
        panic!("one authored original")
    };
    let [settled] = neutral.managed_cleanup_receipts() else {
        panic!("one authentic neutral original")
    };
    assert_ne!(
        cleanup.inspection().run_identity(),
        settled.inspection().run_identity()
    );
    assert_wire(&released, "managed", receipt_fields(&[cleanup]));
    let original_identity = cleanup.inspection().run_identity().to_owned();
    assert_eq!(cleanup.dispatch_posture(), Posture::Pending);
    assert!(!discharged([&*cleanup, settled].into_iter()));
    assert!(!discharged([settled, &*cleanup].into_iter()));
    assert!(transport.0.lock().unwrap().is_empty());
    runtime
        .runtime()
        .resume_original_cleanup_dispatch(cleanup, &scope)
        .unwrap()
        .expect("original first-send result");
    assert_eq!(cleanup.dispatch_posture(), Posture::Observed);
    assert!(!discharged([&*cleanup, settled].into_iter()));
    assert!(!discharged([settled, &*cleanup].into_iter()));
    assert_eq!(cleanup.inspection().run_identity(), original_identity);
    assert_eq!(transport.0.lock().unwrap().len(), 1);
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
fn request_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        WorthQueryAdmissionClock::system(),
    )
}
fn assert_wire(outcome: &Outcome, field: &str, expected: serde_json::Value) {
    let wire = serde_json::to_value(outcome).unwrap();
    assert_eq!(wire[field], expected);
    assert_eq!(&serde_json::from_value::<Outcome>(wire).unwrap(), outcome);
}
fn receipt_fields(receipts: &[&ManagedReceipt]) -> serde_json::Value {
    serde_json::Value::Array(receipts.iter().map(|receipt| {
        let inspection = receipt.inspection();
        serde_json::json!({"logical_run_identity":inspection.logical_run_identity(), "run_identity":inspection.run_identity(), "provider_session_identity":inspection.provider_session_identity(), "resources_released":inspection.resources_released()})
    }).collect())
}
fn assert_collection_wire(receipts: &[&ManagedReceipt], retired_component_count: usize) {
    let descriptions = receipts.iter().map(|r| cleanup_description::describe(r));
    let outcome = Outcome::CollectionReleased {
        request_id: "wire".into(),
        custody_id: "wire".into(),
        sequence: 2,
        retired_component_count,
        managed: descriptions.collect(),
    };
    assert_wire(&outcome, "managed", receipt_fields(receipts));
}
async fn deposit_collection(
    application: &CausalHttpApplication,
    id: &str,
) -> (SourceMutationCustody, CleanupReceipt, CustodyDescription) {
    use super::super::super::super::protocol::BankHttpMutationPosture as Stage;
    use CustodyDescription::{Closed, Completed, Pending, Recorded, Terminal};
    let inspect = |owner: &SourceMutationCustody| match owner.description() {
        Pending(posture, cleanup) => Ok((posture, cleanup)),
        Recorded(receipt) => Err(receipt),
        Completed { .. } => panic!("unexpected Completed"),
        Terminal => panic!("unexpected Terminal"),
        Closed => panic!("unexpected Closed"),
    };
    let rt = &application.runtime;
    let work = std::num::NonZeroUsize::new(128).unwrap();
    let (scope, release_scope) = (request_scope(), request_scope());
    let actor = application.authenticate(&scope).await;
    let request = rt.request(&actor, &scope);
    let scheduled = rt.schedule_next_managed_cleanup_on_worker_for_test();
    let scheduler = scheduled.unwrap();
    let input = parse_deposit("fixture:1", "fixture:100", 1).unwrap();
    let controls = BankMutationControls::new(scope.clone(), BankIdempotencyKey::new(id).unwrap());
    let (mut owner, _) = start(rt, &actor, input, controls, &request, work, None).unwrap();
    drop(scheduler);
    let (posture, cleanup) = inspect(&owner).expect("initial Pending; actual Recorded");
    assert_eq!(posture, Stage::Indeterminate);
    let expected = serde_json::Value::Array(cleanup.iter().map(|failure| serde_json::json!({"run_identity":failure.run_identity, "graph_session_identity":failure.graph_session_identity, "failure":failure.failure})).collect());
    assert_eq!(cleanup.len(), 1, "one executed deposit cleanup failure");
    assert_eq!(serde_json::to_value(&cleanup).unwrap(), expected);
    let fresh = || rt.request(&actor, &release_scope);
    let receipt = owner.release_cleanup(&fresh()).unwrap();
    assert_eq!(receipt.managed_cleanup_receipts().len(), 1);
    let (owner, denial) = owner.advance(&fresh(), work);
    assert!(denial.is_none(), "keyed resolution: {denial:?}");
    let (actual, failures) = inspect(&owner).expect("keyed Pending; actual Recorded");
    let expected = (Stage::RetainedDeliveryPending, 0);
    assert_eq!((actual, failures.len()), expected, "{failures:?}");
    let request = fresh();
    let (owner, denial) = owner.advance(&request, work);
    assert!(denial.is_none(), "retained delivery: {denial:?}");
    let recorded = inspect(&owner).expect_err("delivery Recorded; actual Pending");
    eprintln!("deposit {id} Recorded: {recorded:?}");
    let closed = owner.close(&request);
    let closed = closed.unwrap_or_else(|(_, cause)| panic!("lawful deposit close: {cause:?}"));
    (closed, receipt, Pending(posture, cleanup))
}
#[tokio::test]
async fn plural_cleanup_preserves_all_fields_and_owner_grounded_zero() {
    let application = application(AccountId::new(100).unwrap());
    let (_owner_a, first, pending_a) = deposit_collection(&application, "plural-first").await;
    let (_owner_b, second, pending_b) = deposit_collection(&application, "plural-second").await;
    let retired = first.retired_component_count() + second.retired_component_count();
    let ([first], [second]) = (
        first.managed_cleanup_receipts(),
        second.managed_cleanup_receipts(),
    ) else {
        panic!("one actual receipt from each owner")
    };
    let pair = [first, second];
    assert_ne!(
        first.inspection().run_identity(),
        second.inspection().run_identity()
    );
    assert_eq!(first.dispatch_posture(), Posture::NotDeclared);
    assert_eq!(second.dispatch_posture(), Posture::NotDeclared);
    assert!(discharged(pair.into_iter()));
    assert_collection_wire(&pair, retired);
    let CustodyDescription::Pending(posture, mut cleanup) = pending_a else {
        panic!("first validated Pending projection")
    };
    let CustodyDescription::Pending(_, second_cleanup) = pending_b else {
        panic!("second validated Pending projection")
    };
    let expected = serde_json::json!(cleanup.iter().chain(&second_cleanup).collect::<Vec<_>>());
    cleanup.extend(second_cleanup);
    assert_eq!(cleanup.len(), 2);
    assert_wire(
        &Outcome::Pending {
            request_id: "plural".into(),
            custody_id: "plural".into(),
            sequence: 1,
            posture,
            cleanup,
        },
        "cleanup",
        expected,
    );
    let scope = request_scope();
    let principal = application.authenticate(&scope).await;
    let AdmittedBankHttpMutation::Deposit(input) =
        parse_deposit("fixture:1", "fixture:100", 1).unwrap()
    else {
        panic!("admitted deposit")
    };
    let mut initial = application
        .runtime
        .mutate(bank_server::mutations::deposit(input))
        .as_principal(&principal)
        .controls(BankMutationControls::new(
            scope.clone(),
            BankIdempotencyKey::new("genuine-zero").unwrap(),
        ))
        .retain_operation()
        .unwrap_or_else(|_| panic!("retain unadvanced Initial original"));
    assert_eq!(initial.posture(), WorthQuerySourceOnlyPosture::Initial);
    let release_scope = request_scope();
    let request = application.runtime.request(&principal, &release_scope);
    let empty = initial.release_cleanup(&request).unwrap();
    assert_eq!(empty.managed_cleanup_receipts().len(), 0);
    let closed = initial
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("Initial original closes: {cause:?}"));
    assert_eq!(closed.managed_cleanup_receipts().len(), 0);
    assert!(may_release(&closed, Some(&empty)));
    assert!(may_release(&closed, None));
    assert_collection_wire(&[], empty.retired_component_count());
}
