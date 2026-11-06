use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryBridgeMutationArtifactIdentity, WorthQueryMutationEvidenceDigest};
use worth_runtime_bridge::facade::{
    BridgeMutationAuthorityBundle, BridgeWritebackFailureClass, BridgeWritebackOutcomeClass,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationProvenanceEvidence {
    contract_digest: WorthQueryMutationEvidenceDigest,
    writeback_effect_artifact_digest: WorthQueryMutationEvidenceDigest,
    effect_intent_digest: WorthQueryMutationEvidenceDigest,
    effect_intent_patch_canonical_basis: WorthQueryMutationEvidenceDigest,
    feedback_provenance_digest: WorthQueryMutationEvidenceDigest,
    causality_digest: WorthQueryMutationEvidenceDigest,
    strategy_descriptor_digest: WorthQueryMutationEvidenceDigest,
    execution_record_digest: WorthQueryMutationEvidenceDigest,
    outcome_class: Option<BridgeWritebackOutcomeClass>,
    authoritative_artifact_digest: Option<WorthQueryMutationEvidenceDigest>,
    request_digest: Option<WorthQueryMutationEvidenceDigest>,
    receipt_digest: Option<WorthQueryMutationEvidenceDigest>,
    failure_class: Option<BridgeWritebackFailureClass>,
}

impl WorthQueryMutationProvenanceEvidence {
    pub(in crate::runtime) fn from_bridge(
        bundle: &BridgeMutationAuthorityBundle,
    ) -> Result<Self, IdentityDerivationDenial> {
        let provenance = bundle.provenance();
        Ok(Self {
            contract_digest: imported_artifact(
                "provenance-contract",
                provenance.contract_digest(),
            )?,
            writeback_effect_artifact_digest: imported_artifact(
                "provenance-writeback-effect-artifact",
                provenance.writeback_effect_artifact_digest(),
            )?,
            effect_intent_digest: imported_artifact(
                "provenance-effect-intent",
                provenance.effect_intent_digest(),
            )?,
            effect_intent_patch_canonical_basis: imported_artifact(
                "provenance-effect-intent-patch-canonical-basis",
                provenance.effect_intent_patch_canonical_basis(),
            )?,
            feedback_provenance_digest: imported_artifact(
                "provenance-feedback",
                provenance.feedback_provenance_digest(),
            )?,
            causality_digest: imported_artifact(
                "provenance-causality",
                provenance.causality_digest(),
            )?,
            strategy_descriptor_digest: imported_artifact(
                "provenance-strategy-descriptor",
                provenance.strategy_descriptor_digest(),
            )?,
            execution_record_digest: imported_artifact(
                "provenance-execution-record",
                provenance.execution_record_digest(),
            )?,
            outcome_class: provenance.outcome_class(),
            authoritative_artifact_digest: provenance
                .authoritative_artifact_digest()
                .map(|digest| imported_artifact("provenance-authoritative-artifact", digest))
                .transpose()?,
            request_digest: provenance
                .request_digest()
                .map(|digest| imported_artifact("provenance-request", digest))
                .transpose()?,
            receipt_digest: provenance
                .receipt_digest()
                .map(|digest| imported_artifact("provenance-receipt", digest))
                .transpose()?,
            failure_class: provenance.failure_class(),
        })
    }

    pub fn contract_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.contract_digest
    }

    pub fn writeback_effect_artifact_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.writeback_effect_artifact_digest
    }

    pub fn effect_intent_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.effect_intent_digest
    }

    pub fn effect_intent_patch_canonical_basis(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.effect_intent_patch_canonical_basis
    }

    pub fn feedback_provenance_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.feedback_provenance_digest
    }

    pub fn causality_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.causality_digest
    }

    pub fn strategy_descriptor_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.strategy_descriptor_digest
    }

    pub fn execution_record_digest(&self) -> &WorthQueryMutationEvidenceDigest {
        &self.execution_record_digest
    }

    pub fn outcome_class(&self) -> Option<BridgeWritebackOutcomeClass> {
        self.outcome_class
    }

    pub fn authoritative_artifact_digest(&self) -> Option<&WorthQueryMutationEvidenceDigest> {
        self.authoritative_artifact_digest.as_ref()
    }

    pub fn request_digest(&self) -> Option<&WorthQueryMutationEvidenceDigest> {
        self.request_digest.as_ref()
    }

    pub fn receipt_digest(&self) -> Option<&WorthQueryMutationEvidenceDigest> {
        self.receipt_digest.as_ref()
    }

    pub fn failure_class(&self) -> Option<BridgeWritebackFailureClass> {
        self.failure_class
    }

    #[cfg(test)]
    pub(crate) fn test_only(
        execution_record_digest: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            contract_digest: imported_artifact("provenance-contract", "contract:test")?,
            writeback_effect_artifact_digest: imported_artifact(
                "provenance-writeback-effect-artifact",
                "writeback-effect:test",
            )?,
            effect_intent_digest: imported_artifact(
                "provenance-effect-intent",
                "effect-intent:test",
            )?,
            effect_intent_patch_canonical_basis: imported_artifact(
                "provenance-effect-intent-patch-canonical-basis",
                "effect-intent-patch-basis:test",
            )?,
            feedback_provenance_digest: imported_artifact("provenance-feedback", "feedback:test")?,
            causality_digest: imported_artifact("provenance-causality", "causality:test")?,
            strategy_descriptor_digest: imported_artifact(
                "provenance-strategy-descriptor",
                "strategy:test",
            )?,
            execution_record_digest: imported_artifact(
                "provenance-execution-record",
                execution_record_digest,
            )?,
            outcome_class: None,
            authoritative_artifact_digest: None,
            request_digest: None,
            receipt_digest: None,
            failure_class: None,
        })
    }
}

fn imported_artifact(
    role: &'static str,
    artifact: impl Into<String>,
) -> Result<WorthQueryMutationEvidenceDigest, IdentityDerivationDenial> {
    let artifact = WorthQueryBridgeMutationArtifactIdentity::imported(role, artifact)?;
    Ok(WorthQueryMutationEvidenceDigest::source_identity(
        role,
        artifact.evidence_identity(),
    )?)
}
