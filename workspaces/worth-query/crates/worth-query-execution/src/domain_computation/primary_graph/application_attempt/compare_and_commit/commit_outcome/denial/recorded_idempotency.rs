//! Commit denials for a key whose durable record resolves, but not to a
//! receipt this runtime can return.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl WorthQueryApplicationCommitDenial {
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
