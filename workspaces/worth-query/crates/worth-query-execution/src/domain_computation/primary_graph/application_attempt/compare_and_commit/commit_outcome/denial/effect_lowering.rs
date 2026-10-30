//! Typed refusals decided while lowering a program's effects.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationCommitDenial {
    /// Carries exact missing-observation and unique-value refusals through lowering.
    /// Other refusals keep their existing binding rejection and diagnostic subject.
    pub(in crate::domain_computation::primary_graph::application_attempt) fn effect_lowering_denied(
        denial: &WorthQueryApplicationAttemptDenial,
    ) -> Self {
        if let Some(cause) = denial.unobserved_write() {
            return Self::unobserved_write_denied(cause.clone());
        }
        let kind = match denial.kind() {
            WorthQueryApplicationAttemptDenialKind::UniqueValueTaken => {
                WorthQueryApplicationCommitDenialKind::UniqueValueTaken
            }
            WorthQueryApplicationAttemptDenialKind::UniqueIndexUnavailable => {
                WorthQueryApplicationCommitDenialKind::UniqueIndexUnavailable
            }
            _ => {
                return Self::provider_rejected_with_detail(
                    WorthQueryApplicationCommitDenialStage::ProposalBinding,
                    format!("{:?}: {}", denial.kind(), denial.subject()),
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
