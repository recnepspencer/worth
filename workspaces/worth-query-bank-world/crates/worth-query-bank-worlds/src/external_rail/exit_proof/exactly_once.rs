//! Atomic reservation and immutable-correlation evidence under real TCP races.

pub use std::sync::Arc;

pub use bank_external_rail::{
    dispatch, inquire_admission_count, inquire_completed_effect_count, inquire_completed_notice,
    inquire_notice, inquire_status, LedgerStatus, RailDispatch, RailEffectPayload,
    RailExchangeOutcome, RailRejection,
};
pub use tokio::sync::Barrier;
pub use worth_foundational::facade::BoundaryProtocolVersion;

pub use crate::facade::external_rail::support::{
    attempt_for, correlation_for, notice_payload, spawn_rail, ESTATE, EXPECTED_EFFECT,
    EXPECTED_PROTOCOL_IDENTITY, EXPECTED_PROTOCOL_VERSION, FRAME_TIMEOUT, NOTICE, NOTICE_BOUND,
    SUBJECT,
};

pub const CONCURRENT_DUPLICATES: usize = 16;

pub async fn race_dispatches(
    address: std::net::SocketAddr,
    original: RailDispatch,
    drifted: RailDispatch,
) -> [RailExchangeOutcome; 2] {
    let barrier = Arc::new(Barrier::new(2));
    let original_task = spawn_racing_dispatch(address, original, Arc::clone(&barrier));
    let drifted_task = spawn_racing_dispatch(address, drifted, barrier);
    [
        original_task.await.expect("original dispatch task"),
        drifted_task.await.expect("drifted dispatch task"),
    ]
}

pub fn assert_one_completion_and_one_drift(outcomes: &[RailExchangeOutcome; 2]) {
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| **outcome == RailExchangeOutcome::Completed)
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                **outcome
                    == RailExchangeOutcome::Rejected(RailRejection::CorrelationPayloadMismatch)
            })
            .count(),
        1
    );
}

pub async fn observe_matching_completed_notice(
    address: std::net::SocketAddr,
    correlation: bank_external_rail::RailCorrelation,
) -> bank_external_rail::EstateDeathNotice {
    let completed = inquire_completed_notice(address, correlation.clone(), FRAME_TIMEOUT)
        .await
        .expect("the physical consequence owner answers")
        .expect("exactly one racing request completed");
    let admitted = inquire_notice(address, correlation, FRAME_TIMEOUT)
        .await
        .expect("the idempotency owner answers")
        .expect("exactly one racing request was admitted");
    assert_eq!(completed, admitted);
    completed
}

pub fn spawn_racing_dispatch(
    address: std::net::SocketAddr,
    attempt: RailDispatch,
    barrier: Arc<Barrier>,
) -> tokio::task::JoinHandle<RailExchangeOutcome> {
    tokio::spawn(async move {
        barrier.wait().await;
        dispatch(address, attempt, FRAME_TIMEOUT).await
    })
}

pub async fn assert_exact_single_completion(
    address: std::net::SocketAddr,
    correlation: bank_external_rail::RailCorrelation,
    expected: (u64, u64, u64),
) {
    assert_eq!(
        inquire_status(address, correlation.clone(), FRAME_TIMEOUT)
            .await
            .expect("the idempotency owner answers"),
        LedgerStatus::Completed
    );
    assert_eq!(
        inquire_admission_count(address, FRAME_TIMEOUT)
            .await
            .expect("the admission counter answers"),
        1
    );
    assert_eq!(
        inquire_completed_effect_count(address, FRAME_TIMEOUT)
            .await
            .expect("the physical consequence counter answers"),
        1
    );
    let completed = inquire_completed_notice(address, correlation, FRAME_TIMEOUT)
        .await
        .expect("the physical consequence owner answers")
        .expect("one physical consequence exists");
    assert_eq!(
        (completed.estate(), completed.notice(), completed.subject()),
        expected
    );
}

pub fn payload_for_subject(subject: u64) -> RailEffectPayload {
    let bytes = [
        ESTATE.to_be_bytes(),
        NOTICE.to_be_bytes(),
        subject.to_be_bytes(),
    ]
    .concat();
    RailEffectPayload::new(
        EXPECTED_EFFECT,
        EXPECTED_PROTOCOL_IDENTITY,
        EXPECTED_PROTOCOL_VERSION,
        NOTICE_BOUND,
        bytes,
    )
}
