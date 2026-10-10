use super::*;

#[test]
fn causal_envelope_binds_exact_bridge_records_and_external_authority_references() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal"),
            execution,
        )
        .expect("route should succeed");
    let evaluation = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_branch_head(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            ),
            execution,
        )
        .expect("evaluation should succeed");
    let route_record = runtime
        .diagnostics()
        .route_record_for_route_identity(routed.result().result_summary().route_identity())
        .expect("route record should be retained");
    let historical_record = runtime
        .diagnostics()
        .historical_record_for_record_identity(evaluation.record().record_identity())
        .expect("historical record should be retained");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:changed",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:changed",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            external_reference(
                BridgeCausalEvidenceOwner::Query,
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:changed",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_historical_evaluation_reference(evaluation.record()),
            external_reference(
                BridgeCausalEvidenceOwner::Relational,
                BridgeCausalEvidenceReferenceIdentity::relational_authority(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "relational-authority:commit-causal",
                    ),
                )
                .expect("relational reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalInvalidation,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-invalidation:commit-causal",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalEvaluation,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-evaluation:commit-causal",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalForensicAvailability,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-forensic:commit-causal",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalReplayCursor,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-replay-cursor:commit-causal",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalLineage,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-lineage:commit-causal",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
            external_reference(
                BridgeCausalEvidenceOwner::Signal,
                BridgeCausalEvidenceReferenceIdentity::signal(
                    BridgeCausalEvidenceFamily::SignalProvenance,
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "signal-provenance:commit-causal",
                    ),
                )
                .expect("signal reference identity should be valid"),
            ),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("envelope should assemble");

    assert_eq!(envelope.bindings().len(), 10);
    assert_eq!(envelope.counters().evidence_reference_count(), 10);
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
    assert_eq!(envelope.counters().external_authority_reference_count(), 8);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    assert_retained_route_binding(&envelope, &route_record);
    assert_retained_historical_binding(&envelope, &historical_record);
    assert_signal_reference_binding(
        &envelope,
        BridgeCausalEvidenceFamily::SignalInvalidation,
        "signal-invalidation:commit-causal",
    );
    for (family, identity) in [
        (
            BridgeCausalEvidenceFamily::SignalEvaluation,
            "signal-evaluation:commit-causal",
        ),
        (
            BridgeCausalEvidenceFamily::SignalForensicAvailability,
            "signal-forensic:commit-causal",
        ),
        (
            BridgeCausalEvidenceFamily::SignalReplayCursor,
            "signal-replay-cursor:commit-causal",
        ),
        (
            BridgeCausalEvidenceFamily::SignalLineage,
            "signal-lineage:commit-causal",
        ),
        (
            BridgeCausalEvidenceFamily::SignalProvenance,
            "signal-provenance:commit-causal",
        ),
    ] {
        assert_signal_reference_binding(&envelope, family, identity);
    }
    assert!(!envelope.envelope_for_reporting().is_empty());
}

fn assert_retained_route_binding(
    envelope: &crate::diagnostics::BridgeCausalExplanationEnvelope,
    route_record: &crate::diagnostics::BridgeRouteRecord,
) {
    let route_binding = binding_for(
        envelope.bindings(),
        BridgeCausalEvidenceOwner::RuntimeBridge,
        BridgeCausalEvidenceFamily::BridgeRoute,
        route_record.route_identity().as_str(),
    );
    assert_eq!(
        route_binding.binding_class(),
        BridgeCausalEvidenceBindingClass::RetainedBridgeRecord
    );
    assert_eq!(
        route_binding.retained_record_digest_for_reporting(),
        Some(
            expected_retained_route_digest(
                route_record.route_identity().as_str(),
                route_record.invalidation_identity().as_str(),
                route_record.source_commit().as_str(),
            )
            .as_str()
        )
    );
}

fn assert_retained_historical_binding(
    envelope: &crate::diagnostics::BridgeCausalExplanationEnvelope,
    historical_record: &crate::diagnostics::BridgeCanonicalHistoricalEvaluationRecord,
) {
    let historical_binding = binding_for(
        envelope.bindings(),
        BridgeCausalEvidenceOwner::RuntimeBridge,
        BridgeCausalEvidenceFamily::BridgeHistoricalEvaluation,
        historical_record.record_identity().as_str(),
    );
    assert_eq!(
        historical_binding.retained_record_digest_for_reporting(),
        Some(
            expected_retained_historical_digest(
                historical_record.record_identity().as_str(),
                historical_record
                    .decision_log()
                    .decision_log_identity()
                    .as_str(),
                historical_record
                    .decision_log()
                    .snapshot_identity()
                    .as_str(),
            )
            .as_str()
        )
    );
}

fn assert_signal_reference_binding(
    envelope: &crate::diagnostics::BridgeCausalExplanationEnvelope,
    family: BridgeCausalEvidenceFamily,
    identity: &str,
) {
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceOwner::Signal,
            family,
            identity,
        )
        .binding_class(),
        BridgeCausalEvidenceBindingClass::ExternalAuthorityReference
    );
}

#[test]
fn signal_replay_cursor_reference_denies_runtime_bridge_owner_mismatch() {
    let denial = BridgeCausalEvidenceReference::new(
        BridgeCausalEvidenceOwner::RuntimeBridge,
        BridgeCausalEvidenceFamily::SignalReplayCursor,
        BridgeCausalEvidenceReferenceIdentity::signal(
            BridgeCausalEvidenceFamily::SignalReplayCursor,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "signal-replay-cursor:wrong-owner",
            ),
        )
        .expect("signal reference identity should be valid"),
    )
    .expect_err("signal replay cursor must stay signal-owned");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::EvidenceOwnerMismatch
    );
    assert_eq!(
        denial.supplied_owner(),
        BridgeCausalEvidenceOwner::RuntimeBridge
    );
    assert_eq!(denial.expected_owner(), BridgeCausalEvidenceOwner::Signal);
}
