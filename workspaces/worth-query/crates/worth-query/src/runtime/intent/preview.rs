use super::preview_receipt_identity::{
    preview_intent_admission_identity, preview_intent_receipt_identity,
};
use super::*;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewIntentReceipt {
    intent_name: String,
    strategy_identity: String,
    strategy_version: String,
    canonical_input_digest: String,
    source_lane: WorthQueryIntentSourceLane,
    target_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    basis_evidence: Vec<String>,
    basis_evidence_identity: WorthQueryEvidenceIdentity,
    admission_identity: WorthQueryEvidenceIdentity,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewIntentReceipt {
    pub(in crate::runtime) fn new(
        declaration: &WorthQueryIntentDeclaration,
        effect_policy: WorthQueryEffectPolicy,
        basis_admission: &WorthQueryPreviewBasisAdmission,
        admission: WorthQueryEffectAdmission,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let basis_evidence = basis_admission.evidence().to_vec();
        let canonical_input_digest = declaration.input_digest();
        let basis_evidence_identity =
            preview_intent_basis_evidence_identity(declaration.name(), basis_admission)?;
        let admission_identity = preview_intent_admission_identity(
            declaration,
            effect_policy,
            basis_admission,
            admission,
            &canonical_input_digest,
        )?;
        let receipt_identity = preview_intent_receipt_identity(&admission_identity)?;
        Ok(Self {
            intent_name: declaration.name().to_string(),
            strategy_identity: declaration.strategy_name().to_string(),
            strategy_version: declaration.strategy_version().to_string(),
            canonical_input_digest,
            source_lane: declaration.source_lane(),
            target_lane: declaration.target_lane(),
            effect_policy,
            basis_evidence,
            basis_evidence_identity,
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

    pub fn basis_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_evidence_identity
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

fn preview_intent_basis_evidence_identity(
    intent_name: &str,
    basis_admission: &WorthQueryPreviewBasisAdmission,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewIntentBasisEvidence)
            .field_shape(WorthQueryEvidenceTag::new("intent_name"), intent_name)
            .field_usize(
                WorthQueryEvidenceTag::new("basis_evidence_count"),
                basis_admission.evidence_rows().len(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("basis_admission_identity"),
                basis_admission.admission_identity(),
            )
            .seal()?,
    )
}
