//! Mapping a provider rejection stage into the application commit outcome.

use super::super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};

pub(super) fn denied(
    stage: WorthQueryApplicationCommitDenialStage,
) -> WorthQueryApplicationCommitOutcome {
    WorthQueryApplicationCommitOutcome::Denied(
        WorthQueryApplicationCommitDenial::provider_rejected(stage),
    )
}
