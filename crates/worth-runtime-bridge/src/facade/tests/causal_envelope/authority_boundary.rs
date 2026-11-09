use super::*;

#[test]
fn causal_envelope_denies_external_authority_without_bridge_route_evidence() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:external-only",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:external-only",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            external_reference(
                BridgeCausalEvidenceOwner::Query,
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:external-only",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalInvalidation,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-invalidation:external-only",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
        ],
    )
    .expect("request should be valid before bridge authority assembly");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("external evidence alone must not mint a bridge envelope");
    let crate::facade::BridgeCausalEnvelopeConstructionStop::Denied(denial) = denial else {
        panic!("expected domain denial, got {denial:?}")
    };

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRequiredBridgeRouteEvidence
    );
    assert_eq!(denial.family(), BridgeCausalEvidenceFamily::BridgeRoute);
    assert_eq!(denial.counters().evidence_reference_count(), 2);
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 0);
    assert_eq!(denial.counters().external_authority_reference_count(), 2);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}
#[test]
fn causal_reference_denies_owner_mismatch_before_envelope_assembly() {
    let denial = BridgeCausalEvidenceReference::new(
        BridgeCausalEvidenceOwner::Signal,
        BridgeCausalEvidenceFamily::BridgeRoute,
        BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
            BridgeCausalEvidenceFamily::BridgeRoute,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "route-owned-by-bridge",
            ),
        )
        .expect("bridge reference identity should be valid"),
    )
    .expect_err("owner mismatch should deny");
    let crate::facade::BridgeCausalEnvelopeConstructionStop::Denied(denial) = denial else {
        panic!("expected domain denial, got {denial:?}")
    };

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::EvidenceOwnerMismatch
    );
    assert_eq!(denial.supplied_owner(), BridgeCausalEvidenceOwner::Signal);
    assert_eq!(
        denial.expected_owner(),
        BridgeCausalEvidenceOwner::RuntimeBridge
    );
}
