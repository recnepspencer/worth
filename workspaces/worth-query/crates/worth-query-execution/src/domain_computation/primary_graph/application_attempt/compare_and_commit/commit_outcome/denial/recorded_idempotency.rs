//! Commit denials for a key whose durable record resolves, but not to a
//! receipt this runtime can return.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl WorthQueryApplicationCommitDenial {
    /// A fresh key's conditional definition has no installed predecessor at
    /// the selected product's Signal basis; `detail()` carries Bridge's reason.
    pub(in crate::domain_computation::primary_graph) fn conditional_definition_basis(
        cause: &worth_runtime_bridge::facade::BridgeConditionalDenial,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ProductBasisStale,
            stage: WorthQueryApplicationCommitDenialStage::BridgePlanning,
            detail: Some(std::sync::Arc::from(cause.detail())),
            cause: None,
        }
    }
    pub(in crate::domain_computation) const fn original_effect_released() -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::OriginalEffectReleased,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_foreign_affinity(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyForeignAffinity,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_preparation_busy(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyPreparationBusy,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_intent_unverifiable(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyIntentUnverifiable,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            cause: None,
        }
    }
}
