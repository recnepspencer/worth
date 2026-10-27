//! A real Native envelope cannot borrow legacy descriptor admission.
use super::*;
use crate::domain_computation::evidence_identity::WorthQueryEvidenceScope;
use crate::domain_computation::primary_graph::causal_inspection::{
    identity::CausalInspectionOutcomeIdentity, observation_identity::CausalObservationAnchorDigest,
};
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle;
use worth_runtime_bridge::facade::{
    BridgeCausalInspectionAdmissionSummary as Summary,
    BridgeCausalInspectionAdmissionSummaryKind as Kind,
    BridgeConditionalCausalEnvelopeRequest as Request,
};

#[test]
fn real_native_envelope_requires_fresh_admission_under_both_legacy_paths_and_policies() {
    ownership_oracle::with_genuine_native_observation(|owner, observation| {
        // These are explicitly diagnostic descriptors, never fresh admission.
        let admission: CausalInspectionOutcomeIdentity = descriptor("admission").into();
        let anchor: CausalObservationAnchorDigest = descriptor("anchor").into();
        let query = descriptor("query-observation");
        for kind in [Kind::Admitted, Kind::Advisory] {
            let summary = Summary::for_native_observation(
                kind,
                admission.evidence_identity().bridge_evidence_identity(),
                anchor.evidence_identity().bridge_evidence_identity(),
                query.bridge_evidence_identity(),
            )
            .unwrap();
            let envelope = owner
                .assemble_conditional_causal_envelope(
                    Request::new(summary, observation.clone()).unwrap(),
                )
                .unwrap();
            assert!(envelope.retains_native_observation(observation));
            assert_eq!(
                envelope.bindings()[0].reference_evidence_identity(),
                query.bridge_evidence_identity()
            );
            let error = super::super::validate_bridge_summary(&admission, &anchor, kind, &envelope)
                .unwrap_err();
            assert_eq!(
                error.kind(),
                CausalInspectionMaterializationErrorKind::NativeInspectionAdmissionRequired
            );
            for policy in [
                CausalInspectionMaterializationPolicy::DigestReferenceOnly,
                CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact,
            ] {
                let error =
                    validate_materialization_contract(&query, &[], &envelope, policy).unwrap_err();
                assert_eq!(
                    error.kind(),
                    CausalInspectionMaterializationErrorKind::NativeInspectionAdmissionRequired
                );
            }
        }
    });
}

fn descriptor(label: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationAnchor)
        .field_shape(WorthQueryEvidenceTag::new("legacy-test-descriptor"), label)
        .seal()
}
