pub(crate) use worth_query_bank_testkit_external_rail::facade::tls_sender::*;

#[tokio::test]
async fn only_the_installed_bank_tls_root_can_receive_signed_completion() {
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let trusted_pem = certified.cert.pem();
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certified.cert.der().clone()],
            PrivatePkcs8KeyDer::from(certified.signing_key.serialize_der()).into(),
        )
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let destination = format!(
        "https://localhost:{}/v1/inbound/rail-completions",
        address.port()
    );
    assert!(matches!(
        configuration(destination.clone(), None),
        Err(RailCompletionDeliveryConfigurationDenial::MissingPeerTrust)
    ));
    assert!(matches!(
        configuration(destination.clone(), Some("invalid PEM".to_owned())),
        Err(RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)
    ));
    let bank_signer = SigningKey::from_bytes(&[9; 32]);
    let bank = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut stream = acceptor
            .accept(tcp)
            .await
            .expect("installed root authenticates TLS");
        let body = read_http_body(&mut stream).await;
        let acknowledgement = signed_ack(&body, &bank_signer);
        stream
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    acknowledgement.len(),
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        stream.write_all(&acknowledgement).await.unwrap();
        body
    });
    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(
        configuration(destination.clone(), Some(trusted_pem)).unwrap(),
    )
    .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    assert_eq!(
        dispatch(rail_address, notice_attempt(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    let body = tokio::time::timeout(TIMEOUT, bank).await.unwrap().unwrap();
    assert!(body.starts_with(b"BANK-COMPLETION1"));
    for _ in 0..50 {
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
    rail_task.abort();
    panic!("pinned Bank TLS and signed ACK did not settle the sender");
}

#[tokio::test]
async fn foreign_tls_root_cannot_release_the_rail_sender_obligation() {
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let foreign_pem = generate_simple_self_signed(vec!["localhost".to_owned()])
        .unwrap()
        .cert
        .pem();
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certified.cert.der().clone()],
            PrivatePkcs8KeyDer::from(certified.signing_key.serialize_der()).into(),
        )
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let destination = format!(
        "https://localhost:{}/v1/inbound/rail-completions",
        listener.local_addr().unwrap().port()
    );
    let bank = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        acceptor.accept(tcp).await.is_err()
    });
    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(configuration(destination, Some(foreign_pem)).unwrap())
        .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    assert_eq!(
        dispatch(rail_address, notice_attempt(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    assert!(
        tokio::time::timeout(TIMEOUT, bank).await.unwrap().unwrap(),
        "foreign trust must fail the TLS handshake before any HTTP callback"
    );
    for _ in 0..50 {
        let posture = inquire_completion_delivery_posture(rail_address, TIMEOUT)
            .await
            .unwrap();
        if posture.exhausted == 1 {
            assert_eq!(posture.pending, 1);
            rail_task.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    rail_task.abort();
    panic!("foreign Bank certificate did not leave an exhausted sender obligation");
}
