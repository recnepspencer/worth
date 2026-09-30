//! Commit denials for a key whose durable record resolves, but not to a
//! receipt this runtime can return.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_receipt_not_retained(
        commit: worth_relational::facade::history::CommitId,
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyReceiptNotRetained { commit },
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            custom_invariant: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn idempotency_intent_unverifiable(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::IdempotencyIntentUnverifiable,
            stage: WorthQueryApplicationCommitDenialStage::Idempotency,
            detail: None,
            custom_invariant: None,
        }
    }
}
