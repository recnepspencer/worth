pub(crate) use worth_query_bank_testkit_external_rail::facade::exit_proof::protocol_compatibility::*;

#[tokio::test]
async fn distinct_v1_v2_corpora_coexist_without_cross_version_fallback() {
    let rail = spawn_rail();
    for (scenario, version, maximum, corpus) in [
        ("healthy-v1", 1, 24, V1_CORPUS),
        ("healthy-v2", 2, 32, V2_CORPUS),
    ] {
        let correlation = correlation(scenario);
        assert_eq!(
            dispatch_payload(
                rail.addr,
                correlation.clone(),
                EXPECTED_PROTOCOL_IDENTITY,
                version,
                maximum,
                corpus,
            )
            .await,
            RailExchangeOutcome::Completed
        );
        let notice = inquire_notice(rail.addr, correlation, FRAME_TIMEOUT)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (notice.estate(), notice.notice(), notice.subject()),
            (8101, 8102, 8103)
        );
    }
    assert_eq!(admission_count(rail.addr).await, 2);

    for (scenario, version, maximum, corpus) in [
        ("v1-bytes-as-v2", 2, 32, V1_CORPUS),
        ("v2-bytes-as-v1", 1, 24, V2_CORPUS),
    ] {
        let correlation = correlation(scenario);
        assert_eq!(
            dispatch_payload(
                rail.addr,
                correlation.clone(),
                EXPECTED_PROTOCOL_IDENTITY,
                version,
                maximum,
                corpus,
            )
            .await,
            RailExchangeOutcome::Rejected(RailRejection::MalformedNotice)
        );
        assert_unadmitted(rail.addr, correlation, 2).await;
    }

    let wrong_prefix_correlation = correlation("v2-wrong-prefix");
    let mut wrong_prefix = decode_hex(V2_CORPUS);
    wrong_prefix[0] ^= 0xFF;
    assert_eq!(
        dispatch_bytes(
            rail.addr,
            wrong_prefix_correlation.clone(),
            EXPECTED_PROTOCOL_IDENTITY,
            2,
            32,
            wrong_prefix,
        )
        .await,
        RailExchangeOutcome::Rejected(RailRejection::MalformedNotice)
    );
    assert_unadmitted(rail.addr, wrong_prefix_correlation, 2).await;

    let correlation = correlation("future-v3");
    let outcome = dispatch_payload(
        rail.addr,
        correlation.clone(),
        EXPECTED_PROTOCOL_IDENTITY,
        3,
        32,
        V2_CORPUS,
    )
    .await;
    let RailExchangeOutcome::Rejected(RailRejection::UnsupportedProtocolVersion(unsupported)) =
        outcome
    else {
        panic!("future version must retain its typed unsupported posture: {outcome:?}");
    };
    assert_eq!(unsupported.produced(), BoundaryProtocolVersion::new(3));
    assert_eq!(
        unsupported.posture(),
        BoundaryProtocolUnsupportedVersionPosture::ExceedsWindow
    );
    assert_unadmitted(rail.addr, correlation, 2).await;
}

#[tokio::test]
async fn external_owner_preserves_predating_retired_and_threshold_postures() {
    let v2_only = spawn_rail_with_protocol_support(RailProtocolSupportProfile::V2Only);
    assert_unsupported_v1(
        v2_only.addr,
        "predates-v2-only",
        BoundaryProtocolUnsupportedVersionPosture::PredatesWindow,
    )
    .await;

    let retired = spawn_rail_with_protocol_support(RailProtocolSupportProfile::V1Retired);
    assert_unsupported_v1(
        retired.addr,
        "retired-v1",
        BoundaryProtocolUnsupportedVersionPosture::Retired,
    )
    .await;
    let threshold = correlation("retirement-threshold-v2");
    assert_eq!(
        dispatch_payload(
            retired.addr,
            threshold.clone(),
            EXPECTED_PROTOCOL_IDENTITY,
            2,
            32,
            V2_CORPUS,
        )
        .await,
        RailExchangeOutcome::Completed
    );
    assert_eq!(admission_count(retired.addr).await, 1);
    assert!(inquire_notice(retired.addr, threshold, FRAME_TIMEOUT)
        .await
        .unwrap()
        .is_some());
}

#[tokio::test]
async fn untrusted_frame_deserialization_denies_invalid_protocol_values_before_admission() {
    let rail = spawn_rail();
    for (scenario, field, invalid) in [
        (
            "raw-version-fused-identity",
            "protocol_identity",
            serde_json::json!("bank.estate.death-notification.v1"),
        ),
        (
            "raw-uppercase-identity",
            "protocol_identity",
            serde_json::json!("Bank.estate.death-notification"),
        ),
        ("raw-zero-version", "protocol_version", serde_json::json!(0)),
    ] {
        let correlation = correlation(scenario);
        let mut request = serde_json::json!({
            "Dispatch": serde_json::to_value(RailDispatch {
                correlation: correlation.clone(),
                payload: RailEffectPayload::new(
                    EXPECTED_EFFECT,
                    EXPECTED_PROTOCOL_IDENTITY,
                    BoundaryProtocolVersion::new(1),
                    24,
                    decode_hex(V1_CORPUS),
                ),
            })
            .unwrap()
        });
        request["Dispatch"]["payload"][field] = invalid;
        send_invalid_raw_frame(rail.addr, &request).await;
        assert_unadmitted(rail.addr, correlation, 0).await;
    }
}

#[tokio::test]
async fn production_dispatch_rejects_fault_control_fields() {
    let rail = spawn_rail();
    assert_ne!(rail.addr, rail.test_control_addr);
    let correlation = correlation("fault-field-on-production-wire");
    let mut request = serde_json::json!({
        "Dispatch": serde_json::to_value(RailDispatch {
            correlation: correlation.clone(),
            payload: RailEffectPayload::new(
                EXPECTED_EFFECT,
                EXPECTED_PROTOCOL_IDENTITY,
                BoundaryProtocolVersion::new(1),
                24,
                decode_hex(V1_CORPUS),
            ),
        })
        .unwrap()
    });
    request["Dispatch"]["fault_script"] = serde_json::json!("CommitThenLoseResponse");

    send_invalid_raw_frame(rail.addr, &request).await;
    assert_unadmitted(rail.addr, correlation, 0).await;
}
