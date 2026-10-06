//! Unique-value refusals decided while lowering a program's effects.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationCommitDenial {
    /// Carries a lowering refusal to the commit outcome. Unique-value kinds
    /// keep their meaning and name the field; every other lowering refusal
    /// stays a binding rejection.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn effect_lowering_denied(
        denial: &WorthQueryApplicationAttemptDenial,
    ) -> Self {
        let kind = match denial.kind() {
            WorthQueryApplicationAttemptDenialKind::UniqueValueTaken => {
                WorthQueryApplicationCommitDenialKind::UniqueValueTaken
            }
            WorthQueryApplicationAttemptDenialKind::UniqueIndexUnavailable => {
                WorthQueryApplicationCommitDenialKind::UniqueIndexUnavailable
            }
            _ => {
                return Self::provider_rejected(
                    WorthQueryApplicationCommitDenialStage::ProposalBinding,
                )
            }
        };
        Self {
            kind,
            stage: WorthQueryApplicationCommitDenialStage::EffectLowering,
            detail: Some(denial.subject().into()),
            cause: None,
        }
    }
}
