pub(crate) use worth_query_bank_testkit_external_rail::facade::completion_sender::*;

#[tokio::test]
async fn separate_rail_process_installs_sender_before_dispatch_and_reports_exhaustion() {
    let mut rail = RailProcessHandle::spawn_awaiting_completion_installation(
        worth_query_bank_testkit::facade::binary_path("bank-external-rail"),
        "127.0.0.1:0",
    )
    .unwrap();
    let config = RailCompletionDeliveryConfiguration::new(
        "http://127.0.0.1:1/v1/inbound/rail-completions".into(),
        "bank-process-court".into(),
        "rail-primary".into(),
        1,
        [7; 32],
        SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes(),
        60,
        2,
        1,
        Duration::from_millis(20),
        Duration::from_millis(100),
        None,
    )
    .unwrap();
    rail.install_completion_delivery(&config).unwrap();
    let attempt = notice_attempt();
    assert_eq!(
        dispatch(rail.local_addr(), attempt, TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    for _ in 0..30 {
        let posture = inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
            .await
            .unwrap();
        if posture.exhausted == 1 {
            assert_eq!(posture.pending, 1, "exhaustion retains sender obligation");
            assert_eq!(
                inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
                    .await
                    .unwrap(),
                1
            );
            select_fault(
                rail.test_control_addr(),
                FaultScript::AcknowledgeWithoutCompleting,
                TIMEOUT,
            )
            .await
            .unwrap();
            let mut acknowledged_only = notice_attempt();
            acknowledged_only.correlation =
                RailCorrelation::new("estate-death-notice-rail", [5u8; 32]);
            assert_eq!(
                dispatch(rail.local_addr(), acknowledged_only, TIMEOUT).await,
                RailExchangeOutcome::Acknowledged
            );
            assert_eq!(
                inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
                    .await
                    .unwrap()
                    .pending,
                1
            );
            assert_eq!(
                inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
                    .await
                    .unwrap(),
                1
            );
            let closed = rail
                .close()
                .expect("orderly process close reports sender obligations");
            assert_eq!(closed.pending, 1);
            assert_eq!(closed.exhausted, 1);
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("rail process did not expose unresolved sender exhaustion");
}
