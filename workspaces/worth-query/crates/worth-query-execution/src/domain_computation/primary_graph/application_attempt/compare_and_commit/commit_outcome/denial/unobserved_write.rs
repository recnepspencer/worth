//! Preserve the actual missing-field cause at the effect-lowering boundary.
use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationUnobservedWrite;

impl WorthQueryApplicationCommitDenial {
    pub(super) fn unobserved_write_denied(cause: WorthQueryApplicationUnobservedWrite) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::UnobservedWrite,
            stage: WorthQueryApplicationCommitDenialStage::EffectLowering,
            detail: None,
            cause: Some(Box::new(super::denial_cause::DenialCause::UnobservedWrite(
                cause,
            ))),
        }
    }

    /// The exact attempted write refused during effect lowering.
    pub fn unobserved_write(&self) -> Option<&WorthQueryApplicationUnobservedWrite> {
        match self.cause.as_deref() {
            Some(super::denial_cause::DenialCause::UnobservedWrite(cause)) => Some(cause),
            _ => None,
        }
    }
}
