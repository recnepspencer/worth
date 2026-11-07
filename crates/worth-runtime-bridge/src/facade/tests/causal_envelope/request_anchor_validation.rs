use super::*;

#[test]
fn causal_envelope_request_denies_missing_query_observation_anchor() {
    let denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-query-anchor",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-query-anchor",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![missing_bridge_reference(
            BridgeCausalEvidenceFamily::BridgeRoute,
            "route:missing-query-anchor",
        )],
    )
    .expect_err("bridge assembly request must carry a query observation anchor");
    let crate::facade::BridgeCausalEnvelopeConstructionStop::Denied(denial) = denial else {
        panic!("expected domain denial, got {denial:?}")
    };

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingQueryObservationAnchor
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::QueryObservation
    );
    assert_eq!(denial.supplied_owner(), BridgeCausalEvidenceOwner::Query);
    assert_eq!(denial.expected_owner(), BridgeCausalEvidenceOwner::Query);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_request_denies_multiple_query_observation_anchors() {
    let denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:query-anchor-overclaim",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:query-anchor-overclaim",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:primary",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:overclaim",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgeRoute,
                "route:query-anchor-overclaim",
            ),
        ],
    )
    .expect_err("bridge assembly request must bind exactly one query observation anchor");
    let crate::facade::BridgeCausalEnvelopeConstructionStop::Denied(denial) = denial else {
        panic!("expected domain denial, got {denial:?}")
    };

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::QueryObservationAnchorOverclaim
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::QueryObservation
    );
    assert!(denial
        .reference_identity_for_reporting()
        .starts_with("worth.runtime.bridge.causal-envelope-identity.v1:"));
    assert_ne!(
        denial.reference_identity_for_reporting(),
        "query-observation-anchor-count:2"
    );
    assert_ne!(
        denial.reference_evidence_identity().as_str(),
        "query-observation-anchor-count:2"
    );
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}
