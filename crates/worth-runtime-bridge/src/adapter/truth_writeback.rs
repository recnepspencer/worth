use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TruthWritebackAuthorityErrorTag {}
pub type TruthWritebackAuthorityError = BridgeMessageError<TruthWritebackAuthorityErrorTag>;

pub trait TruthWritebackAuthority: Send + Sync + 'static {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthWritebackRequest {
    family_kind: crate::writeback::BridgeWritebackFamilyKind,
    effect_intent: crate::writeback::BridgeWritebackEffectIntent,
    causality_digest: Arc<str>,
    idempotence_digest: Arc<str>,
    idempotence_class: crate::writeback::BridgeWritebackIdempotenceClass,
    digest: Arc<str>,
}

impl TruthWritebackRequest {
    pub(crate) fn from_evidence(
        contract: &crate::writeback::AdmittedBridgeWritebackContract,
        effect: &crate::writeback::BridgeDerivedWritebackEffect,
        loop_prevention: &crate::writeback::BridgeWritebackLoopPreventionReport,
    ) -> Self {
        let idempotence = loop_prevention.idempotence();
        let canonical_basis = format!(
            "truth-writeback-request|contract={}|effect={}|loop-prevention={}|idempotence={}",
            contract.digest(),
            effect.digest(),
            loop_prevention.digest(),
            idempotence.digest(),
        );
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            family_kind: effect.family_kind(),
            effect_intent: effect.effect_intent().clone(),
            causality_digest: Arc::from(idempotence.causality_digest()),
            idempotence_digest: Arc::from(idempotence.digest()),
            idempotence_class: idempotence.idempotence_class(),
            digest: Arc::from(format!("truth-writeback-request:sha256:{digest:x}")),
        }
    }

    pub fn family_kind(&self) -> crate::writeback::BridgeWritebackFamilyKind {
        self.family_kind
    }

    pub fn effect_intent(&self) -> &crate::writeback::BridgeWritebackEffectIntent {
        &self.effect_intent
    }

    pub fn effect_class(&self) -> crate::writeback::BridgeWritebackEffectClass {
        self.effect_intent.effect_class()
    }

    pub fn causality_digest(&self) -> &str {
        &self.causality_digest
    }

    pub fn idempotence_digest(&self) -> &str {
        &self.idempotence_digest
    }

    pub fn idempotence_class(&self) -> crate::writeback::BridgeWritebackIdempotenceClass {
        self.idempotence_class
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthWritebackReceipt {
    outcome_class: crate::writeback::BridgeWritebackOutcomeClass,
    failure_class: Option<crate::writeback::BridgeWritebackFailureClass>,
    effect_intent: crate::writeback::BridgeWritebackEffectIntent,
    authoritative_artifact_digest: Arc<str>,
    request_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl TruthWritebackReceipt {
    pub fn new(
        outcome_class: crate::writeback::BridgeWritebackOutcomeClass,
        request: &TruthWritebackRequest,
    ) -> Self {
        Self::new_with_failure_class(outcome_class, None, request)
    }

    pub fn new_with_failure_class(
        outcome_class: crate::writeback::BridgeWritebackOutcomeClass,
        failure_class: Option<crate::writeback::BridgeWritebackFailureClass>,
        request: &TruthWritebackRequest,
    ) -> Self {
        let authoritative_artifact_digest = derive_writeback_receipt_authoritative_artifact_digest(
            outcome_class,
            failure_class,
            request,
            None,
        );
        Self::from_derived_authoritative_artifact(
            outcome_class,
            failure_class,
            authoritative_artifact_digest,
            request,
            None,
        )
    }

    pub fn canonical_noop_from_prior_receipt(
        request: &TruthWritebackRequest,
        prior_receipt: &TruthWritebackReceipt,
    ) -> Self {
        Self::from_derived_authoritative_artifact(
            crate::writeback::BridgeWritebackOutcomeClass::CanonicalNoop,
            None,
            Arc::from(prior_receipt.authoritative_artifact_digest()),
            request,
            Some(prior_receipt),
        )
    }

    fn from_derived_authoritative_artifact(
        outcome_class: crate::writeback::BridgeWritebackOutcomeClass,
        failure_class: Option<crate::writeback::BridgeWritebackFailureClass>,
        authoritative_artifact_digest: Arc<str>,
        request: &TruthWritebackRequest,
        prior_receipt: Option<&TruthWritebackReceipt>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "truth-writeback-receipt|request={}|outcome:{outcome_class:?}|failure:{}|authoritative={}|prior-receipt={}",
            request.digest(),
            failure_class
                .map(|class| format!("{class:?}"))
                .unwrap_or_else(|| "none".to_string()),
            authoritative_artifact_digest.as_ref(),
            prior_receipt.map_or("none", TruthWritebackReceipt::digest),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            outcome_class,
            failure_class,
            effect_intent: request.effect_intent().clone(),
            authoritative_artifact_digest,
            request_digest: Arc::from(request.digest().to_owned()),
            canonical_basis,
            digest: Arc::from(format!("truth-writeback-receipt:sha256:{digest:x}")),
        }
    }

    pub fn outcome_class(&self) -> crate::writeback::BridgeWritebackOutcomeClass {
        self.outcome_class
    }

    pub fn failure_class(&self) -> Option<crate::writeback::BridgeWritebackFailureClass> {
        self.failure_class
    }

    pub fn effect_intent(&self) -> &crate::writeback::BridgeWritebackEffectIntent {
        &self.effect_intent
    }

    pub fn authoritative_artifact_digest(&self) -> &str {
        self.authoritative_artifact_digest.as_ref()
    }

    pub(crate) fn share_authoritative_artifact_digest(&self) -> Arc<str> {
        Arc::clone(&self.authoritative_artifact_digest)
    }

    pub fn request_digest(&self) -> &str {
        self.request_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

fn derive_writeback_receipt_authoritative_artifact_digest(
    outcome_class: crate::writeback::BridgeWritebackOutcomeClass,
    failure_class: Option<crate::writeback::BridgeWritebackFailureClass>,
    request: &TruthWritebackRequest,
    prior_receipt: Option<&TruthWritebackReceipt>,
) -> Arc<str> {
    let canonical_basis = format!(
        "truth-writeback-authoritative-artifact|request={}|outcome:{outcome_class:?}|failure:{}|effect-intent={}|effect-intent-basis={}|idempotence={}|prior-receipt={}",
        request.digest(),
        failure_class
            .map(|class| format!("{class:?}"))
            .unwrap_or_else(|| "none".to_string()),
        request.effect_intent().digest(),
        request.effect_intent().patch_canonical_basis(),
        request.idempotence_digest(),
        prior_receipt.map_or("none", TruthWritebackReceipt::digest),
    );
    let digest = Sha256::digest(canonical_basis.as_bytes());
    Arc::from(format!(
        "truth-writeback-authoritative-artifact:sha256:{digest:x}"
    ))
}
