pub(crate) use worth_query_bank_worlds::facade::external_rail::completion_sender::*;

#[tokio::test]
async fn completed_owner_retries_identical_signed_bytes_until_exact_bank_ack() {
    let bank = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bank_address = bank.local_addr().unwrap();
    let rail_signing = SigningKey::from_bytes(&[7; 32]);
    let bank_signing = SigningKey::from_bytes(&[9; 32]);
    let bank_verifying_key = bank_signing.verifying_key().to_bytes();
    let bank_task = tokio::spawn(async move {
        let mut messages = Vec::new();
        for attempt in 0..3 {
            let (mut connection, _) = bank.accept().await.unwrap();
            let message = read_http_body(&mut connection).await;
            assert_signed_meaning(&message, &rail_signing.verifying_key());
            if attempt == 0 {
                reply_chunked_oversized(&mut connection).await;
            } else if attempt == 1 {
                reply(&mut connection, b"unsigned acknowledgement").await;
            } else {
                let acknowledgement = signed_ack(&message, &bank_signing);
                reply(&mut connection, &acknowledgement).await;
            }
            messages.push(message);
        }
        messages
    });

    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(
        RailCompletionDeliveryConfiguration::new(
            format!("http://{bank_address}/v1/inbound/rail-completions"),
            "bank-process-court".into(),
            "rail-primary".into(),
            1,
            [7; 32],
            bank_verifying_key,
            60,
            2,
            3,
            Duration::from_millis(20),
            Duration::from_secs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    let attempt = notice_attempt();
    assert_eq!(
        dispatch(rail_address, attempt.clone(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    assert_eq!(
        dispatch(rail_address, attempt, TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    let messages = tokio::time::timeout(TIMEOUT, bank_task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        messages[0], messages[1],
        "ambiguous ACK resends identical bytes and ID"
    );
    assert_eq!(messages[1], messages[2]);
    assert_eq!(
        inquire_completed_effect_count(rail_address, TIMEOUT)
            .await
            .unwrap(),
        1
    );
    for _ in 0..30 {
        if inquire_completion_delivery_posture(rail_address, TIMEOUT)
            .await
            .unwrap()
            .pending
            == 0
        {
            rail_task.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("authenticated custody ACK did not release the rail obligation");
}

#[tokio::test]
async fn unsigned_http_denial_retries_but_exact_signed_permanent_denial_settles_distinctly() {
    let bank = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bank_address = bank.local_addr().unwrap();
    let bank_signing = SigningKey::from_bytes(&[9; 32]);
    let bank_verifying_key = bank_signing.verifying_key().to_bytes();
    let bank_task = tokio::spawn(async move {
        let (mut first, _) = bank.accept().await.unwrap();
        let original = read_http_body(&mut first).await;
        first
            .write_all(b"HTTP/1.1 409 Conflict\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let (mut second, _) = bank.accept().await.unwrap();
        let retry = read_http_body(&mut second).await;
        assert_eq!(original, retry, "HTTP status cannot settle rail custody");
        let denial = signed_ack_with_posture(&retry, &bank_signing, 4);
        reply(&mut second, &denial).await;
    });

    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(
        RailCompletionDeliveryConfiguration::new(
            format!("http://{bank_address}/v1/inbound/rail-completions"),
            "bank-process-court".into(),
            "rail-primary".into(),
            1,
            [7; 32],
            bank_verifying_key,
            60,
            2,
            2,
            Duration::from_millis(20),
            Duration::from_secs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    assert_eq!(
        dispatch(rail_address, notice_attempt(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    tokio::time::timeout(TIMEOUT, bank_task)
        .await
        .unwrap()
        .unwrap();
    for _ in 0..30 {
        let posture = inquire_completion_delivery_posture(rail_address, TIMEOUT)
            .await
            .unwrap();
        if posture.permanently_denied == 1 {
            assert_eq!(posture.pending, 0);
            assert_eq!(posture.exhausted, 0);
            assert_eq!(
                inquire_completed_effect_count(rail_address, TIMEOUT)
                    .await
                    .unwrap(),
                1
            );
            rail_task.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("signed permanent denial was not classified separately");
}
