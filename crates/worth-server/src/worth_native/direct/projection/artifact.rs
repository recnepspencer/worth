use worth_query::facade::foundation::{
    ConsumedProjectionFactSet, ProjectionConsumptionWarningKind,
    SelfDescribingProjectionConsumptionEnvelope, WorthQueryConsumedProjectionAuthority,
};

use crate::{
    WorthServerDirectContextArtifact, WorthServerQuerySupportPosture, WorthServerResponseEnvelope,
};

use super::{WorthServerDirectMaterializationDigest, WorthServerDirectProjectionFactReceipt};

#[derive(Debug)]
pub struct WorthServerDirectProjection {
    plan_proof: crate::WorthServerOperationPlanProof,
    support_posture: WorthServerQuerySupportPosture,
    workspace_name: String,
    handoff_digest: String,
    direct_context: WorthServerDirectContextArtifact,
    basis_digest: Option<String>,
    policy_digest: String,
    result_shape_digest: String,
    narrowed_result_shape_digest: String,
    facts: ConsumedProjectionFactSet,
    fact_receipt: WorthServerDirectProjectionFactReceipt,
    warning_kinds: Vec<ProjectionConsumptionWarningKind>,
    projection_consumption_envelope: SelfDescribingProjectionConsumptionEnvelope,
    response_envelope: WorthServerResponseEnvelope,
    canonical_digest: String,
}

pub(crate) struct WorthServerDirectProjectionParts {
    pub(crate) plan_proof: crate::WorthServerOperationPlanProof,
    pub(crate) support_posture: WorthServerQuerySupportPosture,
    pub(crate) workspace_name: String,
    pub(crate) handoff_digest: String,
    pub(crate) direct_context: WorthServerDirectContextArtifact,
    pub(crate) authority: WorthQueryConsumedProjectionAuthority,
    pub(crate) warning_kinds: Vec<ProjectionConsumptionWarningKind>,
    pub(crate) response_envelope: WorthServerResponseEnvelope,
}

impl WorthServerDirectProjection {
    pub(crate) fn from_authority(
        parts: WorthServerDirectProjectionParts,
    ) -> Result<Self, worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial>
    {
        let WorthServerDirectProjectionParts {
            plan_proof,
            support_posture,
            workspace_name,
            handoff_digest,
            direct_context,
            authority,
            warning_kinds,
            response_envelope,
        } = parts;
        let materialization_digest = authority
            .receipt()
            .materialized_fact_posture()
            .map(|posture| WorthServerDirectMaterializationDigest::new(posture.posture_digest()))
            .unwrap_or_else(|| {
                WorthServerDirectMaterializationDigest::new(authority.receipt().receipt_digest())
            });
        let fact_receipt = WorthServerDirectProjectionFactReceipt::from_projection_receipt(
            authority.receipt(),
            materialization_digest,
        );
        let basis_digest = authority.contract().basis_digest().map(str::to_string);
        let policy_digest = authority.contract().policy_digest().to_string();
        let result_shape_digest = authority
            .contract()
            .canonical_result_shape_digest()
            .to_string();
        let narrowed_result_shape_digest = authority
            .contract()
            .narrowed_result_shape_digest()
            .to_string();
        let facts = authority.facts().clone();
        let projection_consumption_envelope =
            authority.receipt().projection_consumption_envelope()?;
        let canonical_digest = format!(
            "worth-server-direct-projection-v1:{}:{}:{}",
            handoff_digest,
            fact_receipt.receipt_digest(),
            fact_receipt.materialization_digest().as_str()
        );
        Ok(Self {
            plan_proof,
            support_posture,
            workspace_name,
            handoff_digest,
            direct_context,
            basis_digest,
            policy_digest,
            result_shape_digest,
            narrowed_result_shape_digest,
            facts,
            fact_receipt,
            warning_kinds,
            projection_consumption_envelope,
            response_envelope,
            canonical_digest,
        })
    }

    pub fn plan_proof(&self) -> &crate::WorthServerOperationPlanProof {
        &self.plan_proof
    }

    pub fn support_posture(&self) -> &WorthServerQuerySupportPosture {
        &self.support_posture
    }

    pub fn workspace_name(&self) -> &str {
        &self.workspace_name
    }

    pub fn handoff_digest(&self) -> &str {
        &self.handoff_digest
    }

    pub fn direct_context(&self) -> &WorthServerDirectContextArtifact {
        &self.direct_context
    }

    pub fn basis_digest(&self) -> Option<&str> {
        self.basis_digest.as_deref()
    }

    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    pub fn result_shape_digest(&self) -> &str {
        &self.result_shape_digest
    }

    pub fn narrowed_result_shape_digest(&self) -> &str {
        &self.narrowed_result_shape_digest
    }

    pub fn facts(&self) -> &ConsumedProjectionFactSet {
        &self.facts
    }

    pub fn fact_receipt(&self) -> &WorthServerDirectProjectionFactReceipt {
        &self.fact_receipt
    }

    pub fn materialization_digest(&self) -> &WorthServerDirectMaterializationDigest {
        self.fact_receipt.materialization_digest()
    }

    pub fn warning_kinds(&self) -> &[ProjectionConsumptionWarningKind] {
        &self.warning_kinds
    }

    pub fn projection_consumption_envelope(&self) -> &SelfDescribingProjectionConsumptionEnvelope {
        &self.projection_consumption_envelope
    }

    pub fn response_envelope(&self) -> &WorthServerResponseEnvelope {
        &self.response_envelope
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}
