use super::super::super::identity::CausalInspectionArtifactIdentity;
use super::super::{
    CausalBridgeReadmissionProof, CausalInspectionBoundaryEnvelopeCategory,
    CausalInspectionPerformanceEnvelope, CausalMaterializationReceipt,
};
use super::bridge_backed::QueryCausalEvidenceReferenceArtifact;

pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) struct BuiltBridgeBackedArtifact
{
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) boundary_categories:
        Vec<CausalInspectionBoundaryEnvelopeCategory>,
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) evidence_references:
        Vec<QueryCausalEvidenceReferenceArtifact>,
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) performance:
        CausalInspectionPerformanceEnvelope,
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) receipt:
        CausalMaterializationReceipt,
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) readmission_proof:
        CausalBridgeReadmissionProof,
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) causal_identity:
        CausalInspectionArtifactIdentity,
    pub(in crate::domain_computation::primary_graph::causal_inspection::materialization) artifact_identity:
        CausalInspectionArtifactIdentity,
}
