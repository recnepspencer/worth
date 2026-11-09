use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::{
    WorthQueryAuthorityLane, WorthQueryEffectPolicy, WorthQueryIntentSourceLane,
    WorthQueryPreviewIntentReceipt,
};
use super::intent_receipt_identity::{
    preview_intent_receipt_inspection_basis_identity, preview_intent_receipt_inspection_identity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewIntentReceiptInspection {
    intent_name: String,
    strategy_identity: String,
    strategy_version: String,
    canonical_input_digest: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    basis_evidence: Vec<String>,
    basis_identity: WorthQueryEvidenceIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewIntentReceiptInspection {
    pub(in crate::runtime) fn from_receipt(
        receipt: &WorthQueryPreviewIntentReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence = receipt.basis_evidence().to_vec();
        let basis_identity = preview_intent_receipt_inspection_basis_identity(receipt)?;
        let inspection_identity =
            preview_intent_receipt_inspection_identity(receipt, &basis_identity)?;
        Ok(Self {
            intent_name: receipt.intent_name().to_string(),
            strategy_identity: receipt.strategy_identity().to_string(),
            strategy_version: receipt.strategy_version().to_string(),
            canonical_input_digest: receipt.canonical_input_digest().to_string(),
            source_lane: receipt.source_lane(),
            target_lane: receipt.target_lane(),
            effect_policy: receipt.effect_policy(),
            basis_evidence,
            basis_identity,
            admission_identity: receipt.admission_identity().clone(),
            receipt_identity: receipt.receipt_identity().clone(),
            inspection_identity,
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
    pub fn basis_digest(&self) -> &str {
        self.basis_identity.as_str()
    }
    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }
    pub fn admission_digest(&self) -> &str {
        self.admission_identity.as_str()
    }
    pub fn admission_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admission_identity
    }
    pub fn receipt_digest(&self) -> &str {
        self.receipt_identity.as_str()
    }
    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }
    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}
