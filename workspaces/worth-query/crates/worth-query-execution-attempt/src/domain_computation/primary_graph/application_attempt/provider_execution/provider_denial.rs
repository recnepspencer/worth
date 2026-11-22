//! Mapping a provider rejection stage into the application commit outcome.

use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

pub fn denied(stage: WorthQueryApplicationCommitDenialStage) -> WorthQueryApplicationCommitOutcome {
    WorthQueryApplicationCommitOutcome::Denied(
        WorthQueryApplicationCommitDenial::provider_rejected(stage),
    )
}

pub fn denied_with_detail(
    stage: WorthQueryApplicationCommitDenialStage,
    detail: impl Into<std::sync::Arc<str>>,
) -> WorthQueryApplicationCommitOutcome {
    WorthQueryApplicationCommitOutcome::Denied(
        WorthQueryApplicationCommitDenial::provider_rejected_with_detail(stage, detail),
    )
}
