use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBranchIntentReceipt {
    intent_name: String,
    strategy_identity: String,
    strategy_version: String,
    canonical_input_digest: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    basis_evidence: Vec<String>,
    basis_snapshot_identity: WorthQuerySnapshotIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBranchIntentReceipt {
    pub(in crate::runtime) fn new(
        declaration: &WorthQueryIntentDeclaration,
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryBranchBasisAdmission,
        basis_snapshot_identity: &WorthQuerySnapshotIdentity,
        admission: WorthQueryEffectAdmission,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence = basis_admission.evidence().to_vec();
        let canonical_input_digest = declaration.input_digest();
        let admission_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::BranchIntentAdmission)
                .field_shape(
                    WorthQueryEvidenceTag::new("intent_name"),
                    declaration.name(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("strategy_identity"),
                    declaration.strategy_name(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("strategy_version"),
                    declaration.strategy_version(),
                )
                .field_value(
                    WorthQueryEvidenceTag::new("canonical_input_digest"),
                    &canonical_input_digest,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("source_lane"),
                    declaration.source_lane().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("target_lane"),
                    declaration.target_lane().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("effect_policy"),
                    effect_policy.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("admitted_action"),
                    admission.action().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("admitted_lane"),
                    admission.target_lane().as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("basis_admission_identity"),
                    basis_admission.admission_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("basis_snapshot_identity"),
                    &basis_snapshot_identity.evidence_identity()?,
                )
                .seal()?;
        let receipt_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::BranchIntentReceipt)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("admission_identity"),
                    &admission_identity,
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("posture"),
                    "branch-local-staged-no-authoritative-execution",
                )
                .seal()?;
        Ok(Self {
            intent_name: declaration.name().to_string(),
            strategy_identity: declaration.strategy_name().to_string(),
            strategy_version: declaration.strategy_version().to_string(),
            canonical_input_digest,
            source_lane: declaration.source_lane(),
            target_lane: declaration.target_lane(),
            effect_policy,
            basis_evidence,
            basis_snapshot_identity: basis_snapshot_identity.clone(),
            admission_identity,
            receipt_identity,
        })
    }

    pub fn intent_name(&self) -> &str {
        &self.intent_name
    }

    pub fn strategy_identity(&self) -> &str {
        &self.strategy_identity
    }

    pub fn strategy_version(&self) -> &str {
        &self.strategy_version
    }

    pub fn canonical_input_digest(&self) -> &str {
        &self.canonical_input_digest
    }

    pub fn source_lane(&self) -> WorthQueryIntentSourceLane {
        self.source_lane
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }

    pub fn basis_evidence(&self) -> &[String] {
        &self.basis_evidence
    }

    pub fn basis_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.basis_snapshot_identity
    }

    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }

    pub fn admission_digest(&self) -> &str {
        self.admission_identity.as_str()
    }

    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}
