//! Typed refusals decided while lowering a program's effects.

use super::{
    denial_cause::DenialCause, WorthQueryApplicationCommitDenial,
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationCommitDenial {
    /// Carries exact missing-observation and unique-value refusals through lowering.
    /// Other refusals keep their existing binding rejection and diagnostic subject.
    pub fn effect_lowering_denied(denial: &WorthQueryApplicationAttemptDenial) -> Self {
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
            cause: denial
                .keyed_entity()
                .map(|keyed| Box::new(DenialCause::KeyedEntity(keyed.clone()))),
        }
    }

    pub fn unique_field_index_unavailable(stage: WorthQueryApplicationCommitDenialStage) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::UniqueIndexUnavailable,
            stage,
            detail: None,
            cause: None,
        }
    }

    /// The entity type, unique field and key a `UniqueValueTaken` refusal names.
    pub fn keyed_entity(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryKeyedEntityRefusal> {
        match self.cause.as_deref()? {
            DenialCause::KeyedEntity(keyed) => Some(keyed),
            DenialCause::InvariantExecution(failure) => failure.keyed_entity(),
            _ => None,
        }
    }
}
