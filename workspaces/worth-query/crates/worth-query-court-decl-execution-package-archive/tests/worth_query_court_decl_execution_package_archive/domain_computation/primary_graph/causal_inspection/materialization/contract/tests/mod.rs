//! A real Native envelope cannot borrow legacy descriptor admission.
use worth_query_execution::facade::causal_inspection::CausalInspectionMaterializationErrorKind;
use worth_query_execution::facade::application_inspection::WorthQueryApplicationMaterializationPolicy as CausalInspectionMaterializationPolicy;
use worth_query_execution::facade::evidence_identity::WorthQueryEvidenceIdentity;
use worth_query_execution::facade::evidence_identity::WorthQueryEvidenceTag;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::materialization::contract::validate_materialization_contract;
use worth_query_execution::facade::evidence_identity::WorthQueryEvidenceScope;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::identity::CausalInspectionOutcomeIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::observation_identity::CausalObservationAnchorDigest;
use worth_runtime_bridge::facade::BridgeCausalInspectionAdmissionSummary as Summary;
use worth_runtime_bridge::facade::BridgeCausalInspectionAdmissionSummaryKind as Kind;
use worth_runtime_bridge::facade::BridgeConditionalCausalEnvelopeRequest as Request;

#[test]
fn real_native_envelope_requires_fresh_admission_under_both_legacy_paths_and_policies() {
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::with_genuine_native_observation(|owner, observation| {
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
            let error = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::materialization::validate_bridge_summary(&admission, &anchor, kind, &envelope)
                .unwrap_err();
let error=match error { worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop::Materialization(denial)=>denial, other=>panic!("expected a semantic denial, got {other:?}"), };
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
let error=match error { worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop::Materialization(denial)=>denial, other=>panic!("expected a semantic denial, got {other:?}"), };
                assert_eq!(
                    error.kind(),
                    CausalInspectionMaterializationErrorKind::NativeInspectionAdmissionRequired
                );
            }
        }
    });
}

pub fn descriptor(label: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalObservationAnchor)
        .field_shape(WorthQueryEvidenceTag::new("legacy-test-descriptor"), label)
        .seal()
        .expect("fixture digest must fit")
}

#[test]
fn query_observation_reporting_requires_fresh_admission_under_both_legacy_paths_and_policies() {
    use worth_runtime_bridge::facade::BridgeQueryObservationReportingRequest;
    let admission: CausalInspectionOutcomeIdentity = descriptor("reporting-admission").into();
    let anchor: CausalObservationAnchorDigest = descriptor("reporting-anchor").into();
    let query = descriptor("reporting-original-query");
    let admission_label = admission.evidence_identity().bridge_evidence_identity();
    let anchor_label = anchor.evidence_identity().bridge_evidence_identity();
    let query_label = query.bridge_evidence_identity();
    for kind in [Kind::Admitted, Kind::Advisory] {
        let envelope = BridgeQueryObservationReportingRequest::new(
            kind,
            &admission_label,
            &anchor_label,
            &query_label,
        )
        .unwrap()
        .assemble_with_admission(&mut |_, _| Ok::<_, ()>(()))
        .unwrap();
        let error = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::causal_inspection::materialization::validate_bridge_summary(&admission, &anchor, kind, &envelope)
            .unwrap_err();
        let error=match error { worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop::Materialization(denial)=>denial, other=>panic!("expected a semantic denial, got {other:?}"), };
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
            let error=match error { worth_query_execution::facade::causal_inspection::CausalInspectionConstructionStop::Materialization(denial)=>denial, other=>panic!("expected a semantic denial, got {other:?}"), };
            assert_eq!(
                error.kind(),
                CausalInspectionMaterializationErrorKind::NativeInspectionAdmissionRequired
            );
        }
    }
}
