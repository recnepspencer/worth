use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceTag,
};

use worth_runtime_bridge::facade::BridgeCausalEnvelopeDenial;

use super::super::admission::{AdmittedCausalInspection, AdvisoryCausalInspection};
use super::super::identity::compose_bridge_causal_denial_identity;
use super::super::identity::CausalInspectionOutcomeIdentity;
use super::{
    artifact_identity, policy, CausalInspectionArtifactKind, CausalInspectionMaterializationPolicy,
    CausalInspectionPerformanceEnvelope, CausalInspectionRedactionPolicy,
    CausalMaterializationReceipt, DeniedQueryCausalInspectionArtifact,
    QueryCausalInspectionArtifact, QueryCausalTemporalAsyncExplanation,
};

pub(crate) fn materialize_bridge_denied_admitted_causal_inspection(
    inspection: &AdmittedCausalInspection,
    bridge_denial: &BridgeCausalEnvelopeDenial,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> QueryCausalInspectionArtifact {
    materialize_bridge_denied_query_inspection(
        inspection,
        inspection.admitted_inspection_identity(),
        bridge_denial,
        redaction_policy,
        materialization_policy,
    )
}

pub(crate) fn materialize_bridge_denied_advisory_causal_inspection(
    inspection: &AdvisoryCausalInspection,
    bridge_denial: &BridgeCausalEnvelopeDenial,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> QueryCausalInspectionArtifact {
    materialize_bridge_denied_query_inspection(
        inspection,
        inspection.advisory_inspection_identity(),
        bridge_denial,
        redaction_policy,
        materialization_policy,
    )
}

fn materialize_bridge_denied_query_inspection(
    inspection: impl CausalInspectionBridgeDeniedSubject,
    query_admission_identity: &CausalInspectionOutcomeIdentity,
    bridge_denial: &BridgeCausalEnvelopeDenial,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> QueryCausalInspectionArtifact {
    let denial_reason = "bridge_envelope_denial".to_string();
    let performance = CausalInspectionPerformanceEnvelope::for_bridge_denial(bridge_denial);
    let bridge_denial_identity = compose_bridge_causal_denial_identity(bridge_denial);
    let detail_identity = WorthQueryEvidenceIdentity::compose(
        crate::domain_computation::evidence_identity::WorthQueryEvidenceScope::CausalInspectionDeniedArtifactDetail,
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("query_observation"),
        inspection.subject().query_observation_evidence_identity(),
    )
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("result_shape"),
        inspection
            .subject()
            .result_shape_context_identity()
            .evidence_identity(),
    )
    .field_shape(WorthQueryEvidenceTag::new("reason"), &denial_reason)
    .field_evidence_identity(
        WorthQueryEvidenceTag::new("bridge_denial"),
        &bridge_denial_identity,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("bridge_denial_kind"),
        bridge_denial.kind().as_str(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("bridge_denial_family"),
        bridge_denial.family().as_str(),
    )
    .seal();
    let receipt = CausalMaterializationReceipt::new(
        query_admission_identity,
        None,
        None,
        redaction_policy,
        materialization_policy,
        &performance,
        &detail_identity,
    );
    let artifact_identity = artifact_identity(
        CausalInspectionArtifactKind::Denied,
        query_admission_identity,
        None,
        None,
        &receipt,
        None,
        &detail_identity,
    );
    let temporal_async_explanation = QueryCausalTemporalAsyncExplanation::project(
        inspection.subject().inspection_reason(),
        inspection.subject().observation_outcome(),
        inspection.subject().resolved_evidence_families(),
        Some(bridge_denial.family()),
    );
    QueryCausalInspectionArtifact::Denied(DeniedQueryCausalInspectionArtifact::from_parts(
        query_admission_identity,
        denial_reason,
        inspection.subject().query_observation_identity(),
        inspection.subject().result_shape_context_identity(),
        Some(bridge_denial_identity),
        Some(bridge_denial.kind()),
        Some(bridge_denial.family()),
        temporal_async_explanation,
        policy::boundary_categories(),
        performance,
        receipt,
        artifact_identity,
    ))
}

trait CausalInspectionBridgeDeniedSubject {
    fn subject(&self) -> &super::super::admission_decision::CausalInspectionAdmissionSubject;
}

impl CausalInspectionBridgeDeniedSubject for &AdmittedCausalInspection {
    fn subject(&self) -> &super::super::admission_decision::CausalInspectionAdmissionSubject {
        (*self).subject()
    }
}

impl CausalInspectionBridgeDeniedSubject for &AdvisoryCausalInspection {
    fn subject(&self) -> &super::super::admission_decision::CausalInspectionAdmissionSubject {
        (*self).subject()
    }
}
