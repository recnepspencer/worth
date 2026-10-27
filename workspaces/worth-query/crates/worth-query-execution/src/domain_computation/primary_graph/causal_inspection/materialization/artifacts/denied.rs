use super::super::super::identity::{
    CausalInspectionArtifactIdentity, CausalInspectionOutcomeIdentity,
};
use super::super::super::observation_identity::{
    CausalObservationReceiptIdentity, CausalResultShapeContextIdentity,
};
use super::super::{
    causal_materialization_identity, CausalInspectionArtifactKind,
    CausalInspectionBoundaryEnvelopeCategory, CausalInspectionPerformanceEnvelope,
    CausalMaterializationReceipt, QueryCausalTemporalAsyncExplanation,
};
use crate::domain_computation::evidence_identity::WorthQueryEvidenceIdentity;
use worth_runtime_bridge::facade::{BridgeCausalEnvelopeDenialKind, BridgeCausalEvidenceFamily};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeniedQueryCausalInspectionArtifact {
    query_denial_identity: CausalInspectionOutcomeIdentity,
    query_observation_identity: CausalObservationReceiptIdentity,
    result_shape_context_identity: CausalResultShapeContextIdentity,
    denial_reason: String,
    bridge_denial_identity: Option<WorthQueryEvidenceIdentity>,
    bridge_denial_kind: Option<BridgeCausalEnvelopeDenialKind>,
    bridge_denial_family: Option<BridgeCausalEvidenceFamily>,
    temporal_async_explanation: QueryCausalTemporalAsyncExplanation,
    boundary_categories: Vec<CausalInspectionBoundaryEnvelopeCategory>,
    performance: CausalInspectionPerformanceEnvelope,
    receipt: CausalMaterializationReceipt,
    causal_identity: CausalInspectionArtifactIdentity,
    artifact_identity: CausalInspectionArtifactIdentity,
}

impl DeniedQueryCausalInspectionArtifact {
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) fn from_parts(
        query_denial_identity: &CausalInspectionOutcomeIdentity,
        denial_reason: String,
        query_observation_identity: &CausalObservationReceiptIdentity,
        result_shape_context_identity: &CausalResultShapeContextIdentity,
        bridge_denial_identity: Option<WorthQueryEvidenceIdentity>,
        bridge_denial_kind: Option<BridgeCausalEnvelopeDenialKind>,
        bridge_denial_family: Option<BridgeCausalEvidenceFamily>,
        temporal_async_explanation: QueryCausalTemporalAsyncExplanation,
        boundary_categories: Vec<CausalInspectionBoundaryEnvelopeCategory>,
        performance: CausalInspectionPerformanceEnvelope,
        receipt: CausalMaterializationReceipt,
        artifact_identity: CausalInspectionArtifactIdentity,
    ) -> Self {
        let causal_identity = causal_materialization_identity(
            CausalInspectionArtifactKind::Denied,
            query_denial_identity,
            query_observation_identity.evidence_identity(),
            None,
            None,
        );
        Self {
            query_denial_identity: query_denial_identity.clone(),
            query_observation_identity: query_observation_identity.clone(),
            result_shape_context_identity: result_shape_context_identity.clone(),
            denial_reason,
            bridge_denial_identity,
            bridge_denial_kind,
            bridge_denial_family,
            temporal_async_explanation,
            boundary_categories,
            performance,
            receipt,
            causal_identity,
            artifact_identity,
        }
    }

    pub fn query_denial_for_reporting(&self) -> &str {
        self.query_denial_identity.as_str()
    }

    pub fn query_observation_for_reporting(&self) -> &str {
        self.query_observation_identity.as_str()
    }

    pub(in crate::domain_computation::primary_graph) fn query_observation_identity(
        &self,
    ) -> &CausalObservationReceiptIdentity {
        &self.query_observation_identity
    }

    pub fn result_shape_context_for_reporting(&self) -> &str {
        self.result_shape_context_identity.as_str()
    }

    pub fn denial_reason(&self) -> &str {
        &self.denial_reason
    }

    pub fn bridge_denial_for_reporting(&self) -> Option<&str> {
        self.bridge_denial_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn bridge_denial_kind(&self) -> Option<&str> {
        self.bridge_denial_kind
            .as_ref()
            .map(BridgeCausalEnvelopeDenialKind::as_str)
    }

    pub fn bridge_denial_kind_type(&self) -> Option<BridgeCausalEnvelopeDenialKind> {
        self.bridge_denial_kind
    }

    pub fn bridge_denial_family(&self) -> Option<&str> {
        self.bridge_denial_family
            .as_ref()
            .map(BridgeCausalEvidenceFamily::as_str)
    }

    pub fn bridge_denial_family_type(&self) -> Option<BridgeCausalEvidenceFamily> {
        self.bridge_denial_family
    }

    pub fn temporal_async_explanation(&self) -> &QueryCausalTemporalAsyncExplanation {
        &self.temporal_async_explanation
    }

    pub fn boundary_categories(&self) -> &[CausalInspectionBoundaryEnvelopeCategory] {
        &self.boundary_categories
    }

    pub fn performance(&self) -> &CausalInspectionPerformanceEnvelope {
        &self.performance
    }

    pub fn receipt(&self) -> &CausalMaterializationReceipt {
        &self.receipt
    }

    pub fn causal_identity_for_reporting(&self) -> &str {
        self.causal_identity.as_str()
    }

    pub fn causal_identity(&self) -> &CausalInspectionArtifactIdentity {
        &self.causal_identity
    }

    pub fn artifact_for_reporting(&self) -> &str {
        self.artifact_identity.as_str()
    }

    pub fn artifact_identity(&self) -> &CausalInspectionArtifactIdentity {
        &self.artifact_identity
    }
}
