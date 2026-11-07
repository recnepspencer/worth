use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::CanonicalResultShapeDigest;
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::projection_consumption::ProjectionMaterializedFactPosture;
use crate::runtime::{
    WorthQueryIntentConsumerInspection, WorthQueryIntentExecutionProvenance,
    WorthQueryLiveGraphReadAccessReceipt, WorthQueryRuntimeLiveSubscriptionInstallation,
};

use super::read_receipt_support::materialized_result_digest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLiveReadReceipt {
    view_name: String,
    installation_digest: String,
    installation_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    query_digest: String,
    canonical_query_digest: String,
    canonical_result_shape_digest: CanonicalResultShapeDigest,
    canonical_result_shape_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    subscription_family_digest: String,
    result_digest: String,
    snapshot_identity: WorthQuerySnapshotIdentity,
    snapshot_evidence_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    row_count: usize,
    materialized_fact_posture: Option<ProjectionMaterializedFactPosture>,
    pub(super) live_graph_read_access: Option<WorthQueryLiveGraphReadAccessReceipt>,
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

impl WorthQueryLiveReadReceipt {
    pub(in crate::runtime) fn from_rows(
        installation: &WorthQueryRuntimeLiveSubscriptionInstallation,
        snapshot_identity: WorthQuerySnapshotIdentity,
        materialized_fact_posture: Option<ProjectionMaterializedFactPosture>,
        rows: &[crate::memory_workspace::WorthQueryEntity],
    ) -> Result<Self, IdentityDerivationDenial> {
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        Ok(Self {
            view_name: installation.view_name().to_string(),
            installation_digest: installation.installation_projection().label().to_string(),
            installation_identity: installation.installation_identity().clone(),
            query_digest: installation.query_projection().label().to_string(),
            canonical_query_digest: installation.canonical_query_digest().as_str().to_string(),
            canonical_result_shape_digest: installation.canonical_result_shape_digest().clone(),
            canonical_result_shape_identity: installation.canonical_result_shape_identity().clone(),
            subscription_family_digest: installation
                .subscription_family_projection()
                .label()
                .to_string(),
            result_digest: materialized_result_digest(
                installation.query_projection().label(),
                installation.basis_binding_projection().label(),
                rows,
            )
            .as_str()
            .to_string(),
            snapshot_identity,
            snapshot_evidence_identity,
            row_count: rows.len(),
            materialized_fact_posture,
            live_graph_read_access: None,
            decision_trace_envelope: None,
            execution_provenance: None,
        })
    }

    pub fn view_name(&self) -> &str {
        &self.view_name
    }

    pub fn installation_digest(&self) -> &str {
        &self.installation_digest
    }

    pub fn installation_identity(&self) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.installation_identity
    }

    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn canonical_query_digest(&self) -> &str {
        &self.canonical_query_digest
    }

    pub fn view_shape_digest(&self) -> &str {
        self.canonical_result_shape_digest.as_str()
    }

    pub fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        &self.canonical_result_shape_digest
    }

    pub fn canonical_result_shape_identity(
        &self,
    ) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.canonical_result_shape_identity
    }

    pub fn subscription_family_digest(&self) -> &str {
        &self.subscription_family_digest
    }

    pub fn result_digest(&self) -> &str {
        &self.result_digest
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn snapshot_evidence_identity(
        &self,
    ) -> &crate::evidence_identity::WorthQueryEvidenceIdentity {
        &self.snapshot_evidence_identity
    }

    pub fn row_count(&self) -> usize {
        self.row_count
    }

    pub fn materialized_fact_posture(&self) -> Option<&ProjectionMaterializedFactPosture> {
        self.materialized_fact_posture.as_ref()
    }

    pub fn live_graph_read_access(&self) -> Option<&WorthQueryLiveGraphReadAccessReceipt> {
        self.live_graph_read_access.as_ref()
    }

    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.decision_trace_envelope.as_ref()
    }

    pub fn execution_provenance(&self) -> Option<&WorthQueryIntentExecutionProvenance> {
        self.execution_provenance.as_ref()
    }

    pub fn execution_provenance_chain_digest(&self) -> Option<&str> {
        self.execution_provenance
            .as_ref()
            .map(|provenance| provenance.execution_provenance_chain_digest())
    }

    pub fn consumer_inspection(&self) -> Option<WorthQueryIntentConsumerInspection<'_>> {
        Some(WorthQueryIntentConsumerInspection::from_live_read_receipt(
            self,
        ))
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        view_name: impl Into<String>,
        installation_digest: impl Into<String>,
        installation_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
        query_digest: impl Into<String>,
        canonical_result_shape_digest: CanonicalResultShapeDigest,
        subscription_family_digest: impl Into<String>,
        result_digest: impl Into<String>,
        snapshot_identity: WorthQuerySnapshotIdentity,
        row_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let query_digest = query_digest.into();
        let snapshot_evidence_identity = snapshot_identity.evidence_identity();
        let canonical_result_shape_identity =
            crate::identity::canonical_result_shape_evidence_identity(
                &canonical_result_shape_digest,
            )?;
        Ok(Self {
            view_name: view_name.into(),
            installation_digest: installation_digest.into(),
            installation_identity,
            canonical_query_digest: query_digest.clone(),
            query_digest,
            canonical_result_shape_identity,
            canonical_result_shape_digest,
            subscription_family_digest: subscription_family_digest.into(),
            result_digest: result_digest.into(),
            snapshot_identity,
            snapshot_evidence_identity,
            row_count,
            materialized_fact_posture: None,
            live_graph_read_access: None,
            decision_trace_envelope: None,
            execution_provenance: None,
        })
    }
}
