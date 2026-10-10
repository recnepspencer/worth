use super::mapping_support::{
    bridge_preview_discard_reference, bridge_preview_execution_reference,
    bridge_preview_promotion_reference, bridge_reference, bridge_route_reference,
    missing_bridge_reference, preview_declaration, query_observation_reference,
};
use super::retained_mapping_digest_support::{
    expected_retained_causal_digest, ExpectedRetainedCausalDigestArtifact,
};
use super::{runtime, BridgeRuntimePolicy};
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceBinding, BridgeCausalEvidenceBindingClass, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceOwner, BridgeCausalEvidenceReferenceIdentity, BridgePreviewResidueClass,
    BridgePreviewSessionDeclarationIdentity, BridgeSignalBranchIdentity,
    BridgeSpeculativeBranchBindingIdentity,
};
use crate::speculation::BridgePreviewSessionIdentity;

fn binding_for<'a>(
    bindings: &'a [BridgeCausalEvidenceBinding],
    family: BridgeCausalEvidenceFamily,
    reference_identity: &str,
) -> &'a BridgeCausalEvidenceBinding {
    bindings
        .iter()
        .find(|binding| {
            binding.owner() == BridgeCausalEvidenceOwner::RuntimeBridge
                && binding.family() == family
                && binding.reference_evidence_identity().as_str() == reference_identity
        })
        .expect("expected causal mapping binding should be present")
}
#[test]
fn causal_envelope_maps_retained_preview_records_into_bridge_owned_bindings() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-preview-mapping"),
            execution,
        )
        .expect("route should succeed");
    let discard_admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:causal-discard"),
            preview_declaration(
                BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "preview:causal-discard",
                ),
                BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    "binding:causal-discard",
                ),
                crate::truth_identity_fixtures::truth_branch_fixture("truth:causal-discard"),
                BridgeSignalBranchIdentity::admit_bridge_owned("signal:causal-discard"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:causal-discard"),
            ),
        )
        .expect("discard preview should admit");
    let (discard_active, discard_execution) =
        runtime.activate_preview_session(discard_admitted, 3, 1, 2);
    let (_, discard_record) = runtime
        .discard_preview_session(
            discard_active,
            &discard_execution,
            vec![
                BridgePreviewResidueClass::PreviewExecutionRetained,
                BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
            ],
        )
        .expect("discard should close with zero authoritative residue");
    let promotion_admitted = runtime
        .admit_preview_session(
            BridgePreviewSessionIdentity::admit_bridge_owned("preview-session:causal-promotion"),
            preview_declaration(
                BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "preview:causal-promotion",
                ),
                BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    "binding:causal-promotion",
                ),
                crate::truth_identity_fixtures::truth_branch_fixture("truth:causal-promotion"),
                BridgeSignalBranchIdentity::admit_bridge_owned("signal:causal-promotion"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:causal-promotion"),
            ),
        )
        .expect("promotion preview should admit");
    let (promotion_active, promotion_execution) =
        runtime.activate_preview_session(promotion_admitted, 2, 1, 1);
    let proof = promotion_active.promotion_admissibility_proof();
    let (_, promotion_record) = runtime
        .promote_preview_session(promotion_active, &promotion_execution, &proof)
        .expect("promotion should succeed");

    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:preview-mapping",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:preview-mapping",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:preview-mapping",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_preview_execution_reference(&discard_execution),
            bridge_preview_discard_reference(&discard_record),
            bridge_preview_promotion_reference(&promotion_record),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("preview mappings should assemble");

    assert_eq!(envelope.bindings().len(), 5);
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 4);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 4);
    assert_eq!(envelope.counters().lower_runtime_family_count(), 4);
    assert_eq!(envelope.counters().materialized_detail_count(), 5);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);

    let execution_binding = binding_for(
        envelope.bindings(),
        BridgeCausalEvidenceFamily::BridgePreviewExecution,
        discard_execution.record_identity().as_str(),
    );
    assert_eq!(
        execution_binding.binding_class(),
        BridgeCausalEvidenceBindingClass::RetainedBridgeRecord
    );
    assert_eq!(
        execution_binding.retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::PreviewExecutionRecord,
                &[discard_execution.record_identity().as_str()],
            )
            .as_str()
        )
    );

    let discard_binding = binding_for(
        envelope.bindings(),
        BridgeCausalEvidenceFamily::BridgePreviewDiscard,
        discard_record.record_identity().as_str(),
    );
    assert_eq!(
        discard_binding.retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::PreviewDiscardRecord,
                &[
                    discard_record.record_identity().as_str(),
                    discard_record.preview_execution_record_identity().as_str(),
                ],
            )
            .as_str()
        )
    );

    let promotion_binding = binding_for(
        envelope.bindings(),
        BridgeCausalEvidenceFamily::BridgePreviewPromotion,
        promotion_record.record_identity().as_str(),
    );
    assert_eq!(
        promotion_binding.retained_record_digest_for_reporting(),
        Some(
            expected_retained_causal_digest(
                ExpectedRetainedCausalDigestArtifact::PreviewPromotionRecord,
                &[
                    promotion_record.record_identity().as_str(),
                    promotion_record
                        .preview_execution_record_identity()
                        .as_str(),
                ],
            )
            .as_str()
        )
    );
}

#[test]
fn causal_envelope_denies_missing_preview_mapping_after_required_route_evidence() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-missing-preview"),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-preview",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-preview",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:missing-preview",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgePreviewPromotion,
                "missing-preview-promotion-record",
            ),
        ],
    )
    .expect("request should be valid");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("missing preview mapping should deny");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::BridgePreviewPromotion
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(denial.counters().retained_bridge_binding_count(), 1);
    assert_eq!(denial.counters().missing_bridge_record_count(), 1);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_request_denies_duplicate_preview_references_before_mapping() {
    let duplicate_reference = bridge_reference(
        BridgeCausalEvidenceReferenceIdentity::runtime_bridge(
            BridgeCausalEvidenceFamily::BridgePreviewExecution,
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "preview-execution:duplicate-reference",
            ),
        )
        .expect("bridge reference identity should be valid"),
    );

    let denial = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:duplicate-preview",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:duplicate-preview",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:duplicate-preview",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            duplicate_reference.clone(),
            duplicate_reference,
        ],
    )
    .expect_err("duplicate preview references should deny before mapping");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::DuplicateEvidenceReference
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::BridgePreviewExecution
    );
    assert_eq!(
        denial.reference_identity_for_reporting(),
        "preview-execution:duplicate-reference"
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 0);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}
