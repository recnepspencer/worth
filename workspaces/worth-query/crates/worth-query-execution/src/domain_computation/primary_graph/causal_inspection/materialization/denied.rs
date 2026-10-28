use super::*;

pub fn materialize_denied_causal_inspection(
    inspection: &DeniedCausalInspection,
    bridge_denial: Option<&BridgeCausalEnvelopeDenial>,
    redaction_policy: CausalInspectionRedactionPolicy,
    materialization_policy: CausalInspectionMaterializationPolicy,
) -> QueryCausalInspectionArtifact {
    let denial_reason = inspection
        .decision()
        .violation_kind()
        .map_or("bridge_envelope_denial".to_string(), |kind| {
            kind.as_str().to_string()
        });
    let performance = bridge_denial.map_or_else(
        CausalInspectionPerformanceEnvelope::for_denied_query,
        CausalInspectionPerformanceEnvelope::for_bridge_denial,
    );
    let bridge_denial_identity = bridge_denial.map(compose_bridge_causal_denial_identity);
    let bridge_denial_kind = bridge_denial.map(BridgeCausalEnvelopeDenial::kind);
    let bridge_denial_family = bridge_denial.map(BridgeCausalEnvelopeDenial::family);
    let boundary_categories = policy::boundary_categories();
    let detail_identity = compose_causal_denied_artifact_detail_identity(
        inspection.subject().query_observation_evidence_identity(),
        inspection
            .subject()
            .result_shape_context_identity()
            .evidence_identity(),
        &denial_reason,
        bridge_denial_identity.as_ref(),
        bridge_denial_kind,
        bridge_denial_family,
    );
    let receipt = CausalMaterializationReceipt::new(
        inspection.denied_inspection_identity(),
        None,
        None,
        redaction_policy,
        materialization_policy,
        &performance,
        detail_identity.evidence_identity(),
    );
    let artifact_identity = artifact_identity(
        CausalInspectionArtifactKind::Denied,
        inspection.denied_inspection_identity(),
        None,
        None,
        &receipt,
        None,
        detail_identity.evidence_identity(),
    );
    let temporal_async_explanation =
        project_denied_temporal_async_explanation(inspection, bridge_denial_family);
    QueryCausalInspectionArtifact::Denied(DeniedQueryCausalInspectionArtifact::from_parts(
        inspection.denied_inspection_identity(),
        denial_reason,
        inspection.subject().query_observation_identity(),
        inspection.subject().result_shape_context_identity(),
        bridge_denial_identity,
        bridge_denial_kind,
        bridge_denial_family,
        temporal_async_explanation,
        boundary_categories,
        performance,
        receipt,
        artifact_identity,
    ))
}
