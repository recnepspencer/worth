//! A real rail process outlives the same Home's graceful handoff; its outbox stays native.
use super::installation::{Home, NOTIFICATION_ESTATE, NOTIFICATION_NOTICE};
use bank_domain::{proposals::BankIdempotencyKey, schema::*};
use bank_server::{
    BankAssessedPaymentsApplication, BankAuthenticatedPrincipal, BankHomeRuntime,
    DeathNotificationAction,
};
use std::{sync::Arc, time::Duration};
use worth_query_bank_testkit_server::facade::{
    estate_operations::{
        external_effect_dispatch::rail_transport::{
            BankEstateRailTransport, FaultScript, RailCorrelation,
        },
        freeze_account::fixture::DECEASED,
    },
    support::request_scope,
};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::{WorthQueryCorrectionSourceCoordinates, WorthQueryDispatchOutboxRecord},
};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Pending {
    source: WorthQueryCorrectionSourceCoordinates,
    outbox: OutboxObservation,
    rail: RailCorrelation,
}

/// Passive observations, never an installed inbound contract or completion grant.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OutboxObservation {
    correlation: [u8; 32],
    family: String,
    effect: String,
    protocol: String,
    version: u32,
    maximum_payload_bytes: u64,
    payload: Vec<u8>,
    outcome: u64,
    slot: Option<String>,
}
impl OutboxObservation {
    fn capture(record: &WorthQueryDispatchOutboxRecord) -> Self {
        Self {
            correlation: *record.correlation().bytes(),
            family: record.correlation_family().as_str().to_owned(),
            effect: record.effect().to_owned(),
            protocol: record.protocol_identity().as_str().to_owned(),
            version: record.protocol_version().get(),
            maximum_payload_bytes: record.maximum_payload_bytes(),
            payload: record.payload().to_vec(),
            outcome: record.outcome_identity(),
            slot: record.operation_slot().map(str::to_owned),
        }
    }
}
pub(super) fn connect(
    home: &Home,
    rail: std::net::SocketAddr,
    control: std::net::SocketAddr,
) -> Arc<BankEstateRailTransport> {
    let transport = Arc::new(BankEstateRailTransport::connected_to(rail, control));
    home.runtime()
        .runtime()
        .install_external_effect_transport(transport.clone());
    transport
}
pub(super) fn dispatch_ambiguously(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    transport: &BankEstateRailTransport,
) -> Pending {
    transport.under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let scope = request_scope();
    let request = bank.request(principal, &scope);
    let operation = request
        .mutate(NotifyEstateDeath::new(
            NOTIFICATION_ESTATE,
            NOTIFICATION_NOTICE,
            DECEASED,
        ))
        .without_source()
        .idempotency_owned(key())
        .unwrap_or_else(|(_, _, cause)| panic!("capture real notice: {cause:?}"))
        .retain_durable_operation::<BankAssessedPaymentsApplication, DeathNotificationAction>(
            &request,
            home.runtime(),
        )
        .unwrap_or_else(|(_, cause)| panic!("retain real notice: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request) else {
        panic!("notification must publish through its genuine source owner");
    };
    let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
        panic!("first notification must be fresh");
    };
    let dispatch = receipt
        .external_dispatch()
        .expect("actual transport attempt");
    assert!(
        !dispatch.is_external_completion(),
        "a lost response cannot prove completion"
    );
    assert!(
        dispatch.fault().is_some(),
        "actual ambiguous transport fault must be recorded"
    );
    let pending = Pending {
        source: receipt.correction_source().coordinates(),
        outbox: OutboxObservation::capture(receipt.dispatch_outbox().expect("co-committed outbox")),
        rail: transport.attempts()[0].clone(),
    };
    assert_external_effect(transport, &pending.rail);
    assert!(completed.close(&request).is_ok());
    pending
}
pub(super) fn observe_and_replay(
    home: &Home,
    principal: &BankAuthenticatedPrincipal,
    transport: &BankEstateRailTransport,
    pending: Pending,
) {
    let runtime = home.runtime().runtime();
    let scope = request_scope();
    let bank = BankHomeRuntime::attach(home.runtime()).unwrap();
    let request = bank.request(principal, &scope);
    let operation = request
        .mutate(NotifyEstateDeath::new(
            NOTIFICATION_ESTATE,
            NOTIFICATION_NOTICE,
            DECEASED,
        ))
        .without_source()
        .idempotency_owned(key())
        .unwrap_or_else(|(_, _, cause)| panic!("capture same-key notice: {cause:?}"))
        .retain_durable_operation::<BankAssessedPaymentsApplication, DeathNotificationAction>(
            &request,
            home.runtime(),
        )
        .unwrap_or_else(|(_, cause)| panic!("prepare same-key notice: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request) else {
        panic!("same-key retry must recover its original source receipt");
    };
    let WorthQuerySourceCompletion::Recorded { receipt, .. } = completed.evidence() else {
        panic!("fresh Home must recover the recorded source, never republish it");
    };
    assert_eq!(receipt.commit_reference(), pending.source.relational());
    let outbox = receipt
        .dispatch_outbox()
        .expect("original native outbox is recovered");
    assert_eq!(OutboxObservation::capture(outbox), pending.outbox);
    let observed = runtime
        .with_application_advancement(&scope, |phase| {
            runtime.observe_committed_dispatch_outbox_by_correlation(&phase, outbox.correlation())
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        observed.record(),
        outbox,
        "both fresh native paths recover the same installed outbox"
    );
    assert_eq!(observed.commit_reference(), pending.source.relational());
    drop(observed);
    assert_eq!(receipt.emitted_effect_count(), 1);
    assert!(
        transport.attempts().is_empty(),
        "replay must not dispatch a second effect"
    );
    assert_external_effect(transport, &pending.rail);
    assert!(completed.close(&request).is_ok());
}
fn assert_external_effect(transport: &BankEstateRailTransport, correlation: &RailCorrelation) {
    let notice = transport
        .completed_notice(correlation)
        .expect("rail independently completed the notice");
    assert_eq!(notice.estate(), NOTIFICATION_ESTATE.get());
    assert_eq!(notice.notice(), NOTIFICATION_NOTICE.get());
    assert_eq!(notice.subject(), DECEASED.get());
    assert_eq!(transport.admission_count(), 1);
    assert_eq!(transport.completed_effect_count(), 1);
}
fn key() -> BankIdempotencyKey {
    BankIdempotencyKey::new("h10-death-notification").unwrap()
}
