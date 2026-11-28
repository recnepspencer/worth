use crate::{
    facade::{
        AdmittedBridgeWritebackContract, BridgeWritebackError, BridgeWritebackErrorKind,
        BridgeWritebackIdempotenceBasis, BridgeWritebackStrategyBasis,
        BridgeWritebackStrategyCoherenceDisposition, BridgeWritebackStrategyCoherenceReport,
    },
    writeback::{BridgeDerivedWritebackEffect, BridgeWritebackFeedbackContext},
};

pub(in crate::harness::adapter::adapter_impl) struct AuthorityDenialBoundaryEvidence<'a> {
    pub validation_error: &'a BridgeWritebackError,
    pub unbound_authority: AuthorityDenialBoundaryFailureEvidence<'a>,
    pub unsafe_feedback: AuthorityDenialBoundaryFailureEvidence<'a>,
    pub contradictory_feedback: AuthorityDenialBoundaryFailureEvidence<'a>,
}

pub(in crate::harness::adapter::adapter_impl) struct AuthorityDenialBoundaryFailureEvidence<'a> {
    pub contract: Option<&'a AdmittedBridgeWritebackContract>,
    pub strategy_basis: Option<&'a BridgeWritebackStrategyBasis>,
    pub strategy_coherence: Option<&'a BridgeWritebackStrategyCoherenceReport>,
    pub denial_class: AuthorityDenialBoundaryClass,
    pub failure_kind: BridgeWritebackErrorKind,
    pub failure_digest: Option<&'a str>,
    pub effect: Option<&'a BridgeDerivedWritebackEffect>,
    pub idempotence: Option<&'a BridgeWritebackIdempotenceBasis>,
    pub incoming_feedback_context: Option<&'a BridgeWritebackFeedbackContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::harness::adapter::adapter_impl) enum AuthorityDenialBoundaryClass {
    PreviewValidationDenied,
    UnboundAuthority,
    UnsafeFeedbackPreauthority,
    ContradictoryFeedbackPreauthority,
}

pub(in crate::harness::adapter::adapter_impl) struct AuthorityDenialBoundaryMatrix {
    preview_validation_failure: AuthorityDenialBoundaryFailure,
    unbound_authority_failure: AuthorityDenialBoundaryFailure,
    unsafe_feedback_failure: AuthorityDenialBoundaryFailure,
    contradictory_feedback_failure: AuthorityDenialBoundaryFailure,
}

pub(in crate::harness::adapter::adapter_impl) struct AuthorityDenialBoundaryFailure {
    contract: Option<AdmittedBridgeWritebackContract>,
    strategy_basis: Option<BridgeWritebackStrategyBasis>,
    strategy_coherence: Option<BridgeWritebackStrategyCoherenceReport>,
    denial_class: AuthorityDenialBoundaryClass,
    failure_kind: BridgeWritebackErrorKind,
    failure_digest: Option<String>,
    effect: Option<BridgeDerivedWritebackEffect>,
    idempotence: Option<BridgeWritebackIdempotenceBasis>,
    incoming_feedback_context: Option<BridgeWritebackFeedbackContext>,
}

impl AuthorityDenialBoundaryMatrix {
    pub(super) fn from_boundary_evidence(evidence: AuthorityDenialBoundaryEvidence<'_>) -> Self {
        Self {
            preview_validation_failure: AuthorityDenialBoundaryFailure::preview_validation_failure(
                evidence.validation_error,
            ),
            unbound_authority_failure: AuthorityDenialBoundaryFailure::from_evidence(
                evidence.unbound_authority,
            ),
            unsafe_feedback_failure: AuthorityDenialBoundaryFailure::from_evidence(
                evidence.unsafe_feedback,
            ),
            contradictory_feedback_failure: AuthorityDenialBoundaryFailure::from_evidence(
                evidence.contradictory_feedback,
            ),
        }
    }

    pub(in crate::harness::adapter::adapter_impl) fn preview_validation_failure(
        &self,
    ) -> &AuthorityDenialBoundaryFailure {
        &self.preview_validation_failure
    }

    pub(in crate::harness::adapter::adapter_impl) fn unbound_authority_failure(
        &self,
    ) -> &AuthorityDenialBoundaryFailure {
        &self.unbound_authority_failure
    }

    pub(in crate::harness::adapter::adapter_impl) fn unsafe_feedback_failure(
        &self,
    ) -> &AuthorityDenialBoundaryFailure {
        &self.unsafe_feedback_failure
    }

    pub(in crate::harness::adapter::adapter_impl) fn contradictory_feedback_failure(
        &self,
    ) -> &AuthorityDenialBoundaryFailure {
        &self.contradictory_feedback_failure
    }
}

impl AuthorityDenialBoundaryFailure {
    fn preview_validation_failure(error: &BridgeWritebackError) -> Self {
        Self {
            contract: None,
            strategy_basis: None,
            strategy_coherence: None,
            denial_class: AuthorityDenialBoundaryClass::PreviewValidationDenied,
            failure_kind: error.kind(),
            failure_digest: None,
            effect: None,
            idempotence: None,
            incoming_feedback_context: None,
        }
    }

    fn from_evidence(evidence: AuthorityDenialBoundaryFailureEvidence<'_>) -> Self {
        Self {
            contract: evidence.contract.cloned(),
            strategy_basis: evidence.strategy_basis.cloned(),
            strategy_coherence: evidence.strategy_coherence.cloned(),
            denial_class: evidence.denial_class,
            failure_kind: evidence.failure_kind,
            failure_digest: evidence.failure_digest.map(ToOwned::to_owned),
            effect: evidence.effect.cloned(),
            idempotence: evidence.idempotence.cloned(),
            incoming_feedback_context: evidence.incoming_feedback_context.cloned(),
        }
    }

    pub(in crate::harness::adapter::adapter_impl) fn contract_digest(&self) -> Option<&str> {
        self.contract
            .as_ref()
            .map(AdmittedBridgeWritebackContract::digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn strategy_basis_digest(&self) -> Option<&str> {
        self.strategy_basis
            .as_ref()
            .map(BridgeWritebackStrategyBasis::digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn strategy_coherence_digest(
        &self,
    ) -> Option<&str> {
        self.strategy_coherence
            .as_ref()
            .map(BridgeWritebackStrategyCoherenceReport::digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn strategy_coherence_disposition(
        &self,
    ) -> Option<BridgeWritebackStrategyCoherenceDisposition> {
        self.strategy_coherence
            .as_ref()
            .map(BridgeWritebackStrategyCoherenceReport::disposition)
    }

    pub(in crate::harness::adapter::adapter_impl) fn denial_class(
        &self,
    ) -> AuthorityDenialBoundaryClass {
        self.denial_class
    }

    pub(in crate::harness::adapter::adapter_impl) fn failure_kind(
        &self,
    ) -> BridgeWritebackErrorKind {
        self.failure_kind
    }

    pub(in crate::harness::adapter::adapter_impl) fn failure_digest(&self) -> Option<&str> {
        self.failure_digest.as_deref()
    }

    pub(in crate::harness::adapter::adapter_impl) fn causality_digest(&self) -> Option<&str> {
        self.effect
            .as_ref()
            .map(BridgeDerivedWritebackEffect::causality_digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn writeback_effect_artifact_digest(
        &self,
    ) -> Option<&str> {
        self.effect
            .as_ref()
            .map(BridgeDerivedWritebackEffect::digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn effect_intent_digest(&self) -> Option<&str> {
        self.effect
            .as_ref()
            .map(BridgeDerivedWritebackEffect::effect_intent_digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn effect(
        &self,
    ) -> Option<&BridgeDerivedWritebackEffect> {
        self.effect.as_ref()
    }

    pub(in crate::harness::adapter::adapter_impl) fn idempotence_digest(&self) -> Option<&str> {
        self.idempotence
            .as_ref()
            .map(BridgeWritebackIdempotenceBasis::digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn feedback_provenance_digest(
        &self,
    ) -> Option<&str> {
        self.incoming_feedback_context
            .as_ref()
            .map(BridgeWritebackFeedbackContext::provenance_digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn incoming_feedback_causality_digest(
        &self,
    ) -> Option<&str> {
        self.incoming_feedback_context
            .as_ref()
            .map(BridgeWritebackFeedbackContext::causality_digest)
    }

    pub(in crate::harness::adapter::adapter_impl) fn incoming_feedback_context(
        &self,
    ) -> Option<&BridgeWritebackFeedbackContext> {
        self.incoming_feedback_context.as_ref()
    }
}

impl AuthorityDenialBoundaryClass {
    pub(in crate::harness::adapter::adapter_impl) const fn as_str(self) -> &'static str {
        match self {
            Self::PreviewValidationDenied => "preview-validation-denied",
            Self::UnboundAuthority => "unbound-authority",
            Self::UnsafeFeedbackPreauthority => "unsafe-feedback-preauthority",
            Self::ContradictoryFeedbackPreauthority => "contradictory-feedback-preauthority",
        }
    }
}
