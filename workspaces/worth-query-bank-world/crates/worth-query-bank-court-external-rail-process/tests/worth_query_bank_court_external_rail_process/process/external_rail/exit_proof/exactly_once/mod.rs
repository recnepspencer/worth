pub(crate) use worth_query_bank_worlds::facade::external_rail::exit_proof::exactly_once::*;

#[tokio::test]
async fn concurrent_duplicates_reserve_and_apply_one_physical_effect() {
    let rail = spawn_rail();
    let attempt = attempt_for("concurrent-duplicates");
    let correlation = attempt.correlation.clone();
    let barrier = Arc::new(Barrier::new(CONCURRENT_DUPLICATES));
    let mut tasks = Vec::with_capacity(CONCURRENT_DUPLICATES);

    for _ in 0..CONCURRENT_DUPLICATES {
        let barrier = Arc::clone(&barrier);
        let attempt = attempt.clone();
        let address = rail.addr;
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            dispatch(address, attempt, FRAME_TIMEOUT).await
        }));
    }

    for task in tasks {
        let outcome = task.await.expect("duplicate dispatch task completes");
        assert!(matches!(
            outcome,
            RailExchangeOutcome::Acknowledged | RailExchangeOutcome::Completed
        ));
    }
    assert_exact_single_completion(rail.addr, correlation, (ESTATE, NOTICE, SUBJECT)).await;
}

#[tokio::test]
async fn concurrent_same_key_payload_drift_rejects_the_loser_without_mutating_truth() {
    let rail = spawn_rail();
    let correlation = correlation_for("concurrent-payload-drift");
    let original = RailDispatch {
        correlation: correlation.clone(),
        payload: notice_payload(),
    };
    let drifted = RailDispatch {
        correlation: correlation.clone(),
        payload: payload_for_subject(SUBJECT + 1),
    };
    let outcomes = race_dispatches(rail.addr, original.clone(), drifted.clone()).await;
    assert_one_completion_and_one_drift(&outcomes);
    let completed = observe_matching_completed_notice(rail.addr, correlation.clone()).await;
    assert!(completed.subject() == SUBJECT || completed.subject() == SUBJECT + 1);

    let rejected_retry = if completed.subject() == SUBJECT {
        drifted
    } else {
        original
    };
    assert_eq!(
        dispatch(rail.addr, rejected_retry, FRAME_TIMEOUT).await,
        RailExchangeOutcome::Rejected(RailRejection::CorrelationPayloadMismatch)
    );
    assert_exact_single_completion(
        rail.addr,
        correlation,
        (completed.estate(), completed.notice(), completed.subject()),
    )
    .await;
}

#[tokio::test]
async fn correlation_binds_exact_payload_even_when_another_version_decodes_the_same_notice() {
    let rail = spawn_rail();
    let original = attempt_for("cross-version-drift");
    let correlation = original.correlation.clone();
    assert_eq!(
        dispatch(rail.addr, original, FRAME_TIMEOUT).await,
        RailExchangeOutcome::Completed
    );

    let mut v2_bytes = b"DEATHV2!".to_vec();
    v2_bytes.extend_from_slice(&ESTATE.to_be_bytes());
    v2_bytes.extend_from_slice(&NOTICE.to_be_bytes());
    v2_bytes.extend_from_slice(&SUBJECT.to_be_bytes());
    let alternate_representation = RailDispatch {
        correlation: correlation.clone(),
        payload: RailEffectPayload::new(
            EXPECTED_EFFECT,
            EXPECTED_PROTOCOL_IDENTITY,
            BoundaryProtocolVersion::new(2),
            32,
            v2_bytes,
        ),
    };
    assert_eq!(
        dispatch(rail.addr, alternate_representation, FRAME_TIMEOUT).await,
        RailExchangeOutcome::Rejected(RailRejection::CorrelationPayloadMismatch)
    );
    assert_exact_single_completion(rail.addr, correlation, (ESTATE, NOTICE, SUBJECT)).await;
}
